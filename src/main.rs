use std::ffi::OsString;

use camino::Utf8PathBuf;

use clap::{Parser, Subcommand};

mod ftag;

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
                Ok(_) => println!("Okay"),
                Err(_) => eprintln!("Could not initialize database!"),
            }
        }

        Commands::Tags { path } => match path {
            Some(path) => {
                match ftag::get_file_tags(path) {
                    Ok(tags) => println!("{:?}", tags),
                    Err(_) => todo!(),
                }
            },
            None => ftag::get_global_tags(),
        },

        Commands::Add { path, tags } => ftag::add_tags(path, tags),

        Commands::Rm { path, tags } => ftag::remove_tags(path, tags),
    }
}
