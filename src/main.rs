use std::{io::ErrorKind, ffi::OsString};

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

fn main() {
    let args = Cli::parse();

    // Handle whichever command the user chose
    match args.command {
        Commands::Init => {
            match ftag::init_db() {
                Ok(_) => println!("Initialized database."),
                Err(err) => match err {
                    FtagError::IoError(ErrorKind::AlreadyExists) => eprintln!("Database already exists!"),
                    FtagError::DatabaseError(cause) => eprintln!("Database error: {}!", cause.to_string()),
                    _ => eprintln!("Unexpected error!")
                },
            }
        }

        Commands::Tags { path } => match path {
            Some(path) => {
                match ftag::get_file_tags(&path) {
                    Err(err) => match err {
                        FtagError::IoError(ErrorKind::NotFound) => eprintln!("Filepath {} does not exist!", path),
                        FtagError::DatabaseError(cause) => eprintln!("Database error: {}!", cause.to_string()),
                        _ => eprintln!("Unexpected error!"),
                    }
                    // TODO: print vectors nicely (probably without debug), everywhere
                    Ok(tags) => println!("{:?}", tags),
                }
            },
            None => {
                match ftag::get_global_tags() {
                    Err(err) => match err {
                        FtagError::DatabaseError(cause) => eprintln!("Database error: {}!", cause.to_string()),
                        _ => eprintln!("Unexpected error!"),
                    },
                    Ok(tags) => println!("{:?}", tags),
                }
            },
        },

        Commands::Add { path, tags } => {
            match ftag::add_tags(&path, tags) {
                Ok(_) => todo!(),
                Err(err) => eprintln!("{:?}", err), // TODO: handle error properly
            }
        },

        Commands::Rm { path, tags } => {
            match ftag::remove_tags(&path, tags) {
                Ok(_) => todo!(),
                Err(_) => todo!(),
            }
        }
    }
}
