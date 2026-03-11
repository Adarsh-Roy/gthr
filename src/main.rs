use anyhow::Result;
use clap::Parser;
use gthr::cli::{Cli, Commands};
use gthr::config::settings::Settings;
use gthr::git;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let mut cli = Cli::parse();

    // Handle remote repo cloning (must happen before settings load)
    let _remote = if let Some(ref url) = cli.url {
        let remote = git::prepare_remote_repo(url, cli.keep.as_deref())?;
        cli.root = remote.root.clone();
        Some(remote)
    } else {
        None
    };

    let settings = Settings::load_with_local(&cli.root);

    match cli.command.as_ref().unwrap_or(&Commands::Interactive) {
        Commands::Interactive => gthr::run_interactive_mode(&cli, &settings).await?,
        Commands::Direct => gthr::run_direct_mode(&cli, &settings).await?,
    }

    Ok(())
}
