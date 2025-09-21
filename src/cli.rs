use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "ctx")]
#[command(about = "A CLI tool for directory text ingestion with fuzzy finder capabilities")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Root directory to process
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Pre-include all files and directories
    #[arg(short, long, conflicts_with = "exclude_all")]
    pub include_all: bool,

    /// Pre-exclude all files and directories (pick what to include)
    #[arg(short, long, conflicts_with = "include_all")]
    pub exclude_all: bool,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Skip interactive mode and use current selection
    #[arg(long)]
    pub non_interactive: bool,

    /// Respect .gitignore files
    #[arg(long, default_value = "true")]
    pub respect_gitignore: bool,

    /// Maximum file size to include (in bytes)
    #[arg(long, default_value = "1048576")] // 1MB default
    pub max_file_size: u64,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the interactive fuzzy finder interface
    Interactive,
    /// Generate text ingest directly without interaction
    Direct {
        /// Pattern to include files (glob pattern)
        #[arg(long)]
        include: Vec<String>,
        /// Pattern to exclude files (glob pattern)
        #[arg(long)]
        exclude: Vec<String>,
    },
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            command: Some(Commands::Interactive),
            root: PathBuf::from("."),
            include_all: false,
            exclude_all: false,
            output: None,
            non_interactive: false,
            respect_gitignore: true,
            max_file_size: 1048576,
        }
    }
}