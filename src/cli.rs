use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gthr")]
#[command(about = "A CLI tool for directory text ingestion with fuzzy finder capabilities")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Root directory to process
    #[arg(short, long, default_value = ".")]
    pub root: PathBuf,

    /// Pre-include all files and directories
    #[arg(short = 'I', long = "include-all", conflicts_with = "exclude_all")]
    pub include_all: bool,

    /// Pre-exclude all files and directories (pick what to include)
    #[arg(short = 'E', long = "exclude-all", conflicts_with = "include_all")]
    pub exclude_all: bool,

    /// Pattern to include files (glob pattern)
    #[arg(short = 'i', long = "include")]
    pub include: Vec<String>,

    /// Pattern to exclude files (glob pattern)
    #[arg(short = 'e', long = "exclude")]
    pub exclude: Vec<String>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Respect .gitignore files
    #[arg(long = "respect-gitignore", short = 'g', action = clap::ArgAction::Set, default_value = "true")]
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
    Direct,
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            command: Some(Commands::Interactive),
            root: PathBuf::from("."),
            include_all: false,
            exclude_all: false,
            include: Vec::new(),
            exclude: Vec::new(),
            output: None,
            respect_gitignore: true,
            max_file_size: 1048576,
        }
    }
}
