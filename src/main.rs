use std::{path::Path, ffi::OsString};

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

    /// List tags overall or of a file
    Tags {
        /// Target file
        file: Option<OsString>,
    },

    /// Add tags to a file
    #[command(arg_required_else_help = true)]
    Add {
        /// Target file
        file: OsString,

        /// Tags to add
        #[arg(required = true)]
        tags: Vec<OsString>,
    },

    /// Remove tags from a file
    #[command(arg_required_else_help = true)]
    Rm {
        /// Target file
        file: OsString,

        /// Tags to remove
        #[arg(required = true)]
        tags: Vec<OsString>,
    },
}


fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Init => todo!("Init database"),
        Commands::Tags { file } => {
            match file {
                Some(name) => todo!("File tags {:?}", name),
                None => todo!("Overall tags"),
            }
        }
        Commands::Add { file, tags } => todo!("Add tags {:?}, {:?}", file, tags),
        Commands::Rm { file, tags } => todo!("Remove {:?}, {:?}", file, tags),
    }
}