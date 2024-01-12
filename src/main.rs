use std::{io::ErrorKind, collections::HashSet};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

mod ftag;
use ftag::{FtagError, get_file_tags};
use itertools::Itertools;

/// Utility to tag files for easy access
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize the database
    Init,

    /// List tags of a path or globally
    List {
        /// Target path for list. If unspecified, will list tags globally
        path: Option<Utf8PathBuf>,

        /// Reverse sorting order
        #[arg(short, long)]
        reverse: bool,

        /// Display tag counts on global list (only on global list)
        #[arg(short, long)]
        count: bool,

        /// Sort by descending count, instead of alphabetically (only on global list)
        #[arg(short, long)]
        sortcount: bool,
    },

    /// Add tags to a path
    #[command(arg_required_else_help = true)]
    Add {
        /// Target path
        path: Utf8PathBuf,

        /// Tags to add
        #[arg(required = true)]
        tags: Vec<String>,
    },

    /// Remove tags from a path
    #[command(arg_required_else_help = true)]
    Rm {
        /// Target path
        path: Utf8PathBuf,

        /// Tags to remove
        #[arg(required = true)]
        tags: Vec<String>,
    },

    /// Find files with particular tags
    #[command(arg_required_else_help = true)]
    Find {
        /// Tags that matching files must have
        #[arg(required=true)]
        find: Vec<String>,

        // Display tags of each file
        #[arg(short, long)]
        tags: bool,

        /// Optional tags which matching files must not have
        #[arg(required=false, last=true)]
        exclude: Vec<String>,
    },

    /// Rename a single tag for a path
    #[command(arg_required_else_help = true)]
    Rename {
        /// Target path
        path: Utf8PathBuf,

        /// Old tag name
        #[arg(name="OLD")]
        old_tag: String,

        /// New tag name
        #[arg(name="NEW")]
        new_tag: String,
    }
}

fn display_tags(tags: HashSet<String>, reverse: bool) {
    // Get the HashSet as a vector and alphabetize it
    let mut tags: Vec<_> = tags.into_iter().collect();
    tags.sort(); // alphabetic and case-sensitive

    if reverse {
        tags.reverse();
    }

    // Print them out with a little header 
    for tag in tags {
        println!("{}", tag);
    }
}

fn main() {
    let args = Cli::parse();

    // Handle whichever command the user chose
    match args.command {
        Commands::Init => {
            match ftag::init_db() {
                Ok(_) => println!("Initialized database."),
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::AlreadyExists) => eprintln!("Database already exists!"),
                    _ => eprintln!("{}", err.to_string())
                },
            }
        }

        Commands::List { path, reverse, count, sortcount} => match path {
            Some(path) => {
                match ftag::get_file_tags(&path) {
                    Err(err) => match err {
                        FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                        _ => eprintln!("{}", err.to_string())
                    }
                    Ok(tags) => display_tags(tags, reverse),
                }
            },
            None => {
                match ftag::get_global_tags() {
                    Err(err) => eprintln!("{}", err.to_string()),
                    Ok(tag_counts) => {
                        // Collect the keys and value into a vector of tuples
                        let mut pairs: Vec<(String, u32)> = tag_counts.into_iter().collect();

                        // Sort either by counts or alphabetically
                        if sortcount {
                            // Sort by count, descending order
                            pairs.sort_by(|a, b| b.1.cmp(&a.1));
                        } else {
                            // Sort alphabetically, ascending order
                            pairs.sort_by(|a, b| a.0.cmp(&b.0));
                        }

                        // Reverse the list if that was specified
                        if reverse {
                            pairs.reverse();
                        }

                        for pair in pairs {
                            // If printing counts, put "(#) " on the same line
                            if count {
                                print!("({}) ", pair.1);
                            }

                            // Always print path
                            println!("{}", pair.0);
                        }
                    },
                }
            },
        },

        Commands::Add { path, tags } => {
            match ftag::add_tags(&path, tags) {
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                    _ => eprintln!("{}", err.to_string()),
                },
                Ok(new_tags) => display_tags(new_tags, false),
            }
        },

        Commands::Rm { path, tags } => {
            match ftag::remove_tags(&path, tags) {
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                    _ => eprintln!("{}", err.to_string()),
                },
                Ok(new_tags) => display_tags(new_tags, false),
            }
        },

        Commands::Find { find, exclude , tags } => {
            match ftag::find_tags(&find, &exclude) {
                Err(err) => eprintln!("{}", err.to_string()),
                Ok(mut files) => {
                    // Alphabetize the vector returned
                    files.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

                    // Print them out with a little header 
                    for (file, file_tags) in files {
                        println!("{}", file);
                        if tags {
                            println!("  {}", file_tags.iter().format("; "));
                        }
                    }
                },
            }
        },

        Commands::Rename { path, old_tag, new_tag} => { 
            // Determine whether the path contains old_tag
            let current_tags = get_file_tags(&path);
            match current_tags {
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                    _ => eprintln!("{}", err.to_string()),
                },
                Ok(tags) => {
                    if !tags.contains(&old_tag) {
                        eprintln!("Tag {} not found.", old_tag);
                        return;
                    }

                    // Remove the old tag and swap in the new one
                    if let Err(err) = ftag::remove_tags(&path, vec![old_tag]) {
                        eprintln!("{}", err.to_string());
                    }
                    if let Err(err) = ftag::add_tags(&path, vec![new_tag]) {
                        eprintln!("{}", err.to_string());
                    }

                    // Print out the properly updated tags
                    match ftag::get_file_tags(&path) {
                        Err(err) => eprintln!("{}", err.to_string()),
                        Ok(tags) => display_tags(tags, false),
                    }
                }
            }

        }
    }
}
