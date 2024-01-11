use std::{io::ErrorKind, ffi::OsString, collections::HashSet, arch::is_aarch64_feature_detected};

use camino::Utf8PathBuf;

use clap::{Parser, Subcommand};

mod ftag;

use ftag::FtagError;

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
    Tags {
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
        tags: Vec<OsString>,
    },

    /// Remove tags from a path
    #[command(arg_required_else_help = true)]
    Rm {
        /// Target path
        path: Utf8PathBuf,

        /// Tags to remove
        #[arg(required = true)]
        tags: Vec<OsString>,
    },
}

fn display_tags(name: String, tags: HashSet<String>) {
    // Get the HashSet as a vector and alphabetize it
    let mut tags: Vec<_> = tags.into_iter().collect();
    tags.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    // Print them out with a little header 
    println!("{} tags:", name);
    for tag in tags {
        println!("  {}", tag);
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

        Commands::Tags { path } => match path {
            Some(path) => {
                match ftag::get_file_tags(&path) {
                    Err(err) => match err {
                        FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                        _ => eprintln!("{}", err.to_string())
                    }
                    Ok(tags) => display_tags(path.to_string(), tags),
                }
            },
            None => {
                match ftag::get_global_tags() {
                    Err(err) => eprintln!("{}", err.to_string()),
                    Ok(tags) => display_tags("all file".to_string(), tags),
                }
            },
        },

        Commands::Add { path, tags } => {
            match ftag::add_tags(&path, tags) {
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                    _ => eprintln!("{}", err.to_string()),
                },
                Ok(new_tags) => display_tags(path.to_string(), new_tags),
            }
        },

        Commands::Rm { path, tags } => {
            match ftag::remove_tags(&path, tags) {
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                    _ => eprintln!("{}", err.to_string()),
                },
                Ok(new_tags) => display_tags(path.to_string(), new_tags),
            }
        }
    }
}
