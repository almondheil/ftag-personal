use std::{ffi::OsString, path::PathBuf, process};

use clap::{Parser, Subcommand};

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
        path: Option<PathBuf>,
    },

    /// Add tags to a path
    #[command(arg_required_else_help = true)]
    Add {
        /// Target path
        path: PathBuf,

        /// Tags to add
        #[arg(required = true)]
        tags: Vec<OsString>,
    },

    /// Remove tags from a path
    #[command(arg_required_else_help = true)]
    Rm {
        /// Target path
        path: PathBuf,

        /// Tags to remove
        #[arg(required = true)]
        tags: Vec<OsString>,
    },
}

fn main() {
    let args = Cli::parse();

    // Handle whichever command the user chose
    match args.command {
        Commands::Init => todo!("Init database"),

        Commands::Tags { path } => match path {
            Some(path) => {
                // Make sure the provided arg is a valid path
                if !path.exists() {
                    eprintln!("Path `{}` does not exist!", path.display());
                    process::exit(2);
                }

                // List the tags of that path
                todo!("Single tags for {}", path.display());
            }
            None => todo!("Database tags"),
        },

        Commands::Add { path, tags } => {
            // Make sure the provided arg is a valid path
            if !path.exists() {
                eprintln!("Path `{}` does not exist!", path.display());
                process::exit(2);
            }

            todo!("Add tags {:?}, {:?}", path, tags);
        }

        Commands::Rm { path, tags } => {
            // Make sure the provided arg is a valid path
            if !path.exists() {
                eprintln!("Path `{}` does not exist!", path.display());
                process::exit(2);
            }

            todo!("Remove tags {:?}, {:?}", path, tags);
        }
    }
}
