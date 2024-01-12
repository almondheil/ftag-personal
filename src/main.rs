use std::{io::ErrorKind, collections::HashSet};
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

mod ftag;
use ftag::{FtagError, get_file_tags};

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

    /// List tags of a path or whole database
    List {
        /// Target path
        path: Option<Utf8PathBuf>,
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
        #[arg(required = true)]
        tags: Vec<String>,
    },

    /// Rename a single tag for a path
    #[command(arg_required_else_help = true)]
    Rename {
        /// Target path
        path: Utf8PathBuf,

        /// Old tag name
        old_tag: String,

        /// New tag name
        new_tag: String,
    }
}

fn display_tags(tags: HashSet<String>) {
    // Get the HashSet as a vector and alphabetize it
    let mut tags: Vec<_> = tags.into_iter().collect();
    tags.sort(); // alphabetic and case-sensitive

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

        Commands::List { path } => match path {
            Some(path) => {
                match ftag::get_file_tags(&path) {
                    Err(err) => match err {
                        FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                        _ => eprintln!("{}", err.to_string())
                    }
                    Ok(tags) => display_tags(tags),
                }
            },
            None => {
                match ftag::get_global_tags() {
                    Err(err) => eprintln!("{}", err.to_string()),
                    Ok(tags) => display_tags(tags),
                }
            },
        },

        Commands::Add { path, tags } => {
            match ftag::add_tags(&path, tags) {
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                    _ => eprintln!("{}", err.to_string()),
                },
                Ok(new_tags) => display_tags(new_tags),
            }
        },

        Commands::Rm { path, tags } => {
            match ftag::remove_tags(&path, tags) {
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                    _ => eprintln!("{}", err.to_string()),
                },
                Ok(new_tags) => display_tags(new_tags),
            }
        },

        Commands::Find { tags } => {
            match ftag::find_tags(&tags) {
                Err(err) => eprintln!("{}", err.to_string()),
                Ok(mut files) => {
                    // Alphabetize the vector returned
                    files.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

                    // Print them out with a little header 
                    for file in files {
                        println!("{}", file);
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
                        Ok(tags) => display_tags(tags),
                    }
                }
            }

        }
    }
}
