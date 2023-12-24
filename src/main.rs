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
        file: Option<String>,
    },

    /// Add tags to a file
    #[command(arg_required_else_help = true)]
    Add {
        /// Target file
        file: String,

        /// Tags to add
        #[arg(required = true)]
        tags: Vec<String>,
    },

    /// Remove tags from a file
    #[command(arg_required_else_help = true)]
    Rm {
        /// Target file
        file: String,

        /// Tags to remove
        #[arg(required = true)]
        tags: Vec<String>,
    },
}

fn main() {
    let args = Cli::parse();

    dbg!(args);
}