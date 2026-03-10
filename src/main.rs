use anyhow::Result;
use clap::Parser;
use gthr::cli::{Cli, Commands};
use gthr::config::settings::Settings;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let settings = Settings::load();

    match cli.command.as_ref().unwrap_or(&Commands::Interactive) {
        Commands::Interactive => gthr::run_interactive_mode(&cli, &settings).await?,
        Commands::Direct => gthr::run_direct_mode(&cli, &settings).await?,
    }

    Ok(())
}
