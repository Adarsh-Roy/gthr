mod cli;
mod directory;
mod ui;
mod fuzzy;
mod output;
mod config;

use clap::Parser;
use cli::{Cli, Commands};
use directory::traversal::DirectoryTraverser;
use ui::app::{App, AppMode};
use ui::events::{EventHandler, AppEvent, handle_key_event, AppAction};
use ui::interface::draw_ui;
use output::writer::OutputWriter;
use output::formatter::OutputFormatter;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.as_ref().unwrap_or(&Commands::Interactive) {
        Commands::Interactive => {
            if cli.non_interactive {
                run_non_interactive_mode(&cli).await?;
            } else {
                run_interactive_mode(&cli).await?;
            }
        }
        Commands::Direct { include, exclude } => {
            run_direct_mode(&cli, include, exclude).await?;
        }
    }

    Ok(())
}

async fn run_interactive_mode(cli: &Cli) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create application state
    let traverser = DirectoryTraverser::new(cli.respect_gitignore, cli.max_file_size, cli.include_all);
    let tree = traverser.traverse(&cli.root)?;
    let mut app = App::new(tree);

    let event_handler = EventHandler::new();
    let result = run_app(&mut terminal, &mut app, &event_handler, cli).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: &EventHandler,
    cli: &Cli,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        if app.should_quit {
            break;
        }

        if let Some(event) = event_handler.next_event(Duration::from_millis(50))? {
            match event {
                AppEvent::Key(key_event) => {
                    if app.mode == AppMode::Help {
                        app.set_mode(AppMode::Main);
                        continue;
                    }

                    if let Some(action) = handle_key_event(key_event) {
                        match action {
                            AppAction::Escape => app.handle_escape(),
                            AppAction::Generate => {
                                generate_output(&app.tree, cli)?;
                                app.quit();
                            }
                            AppAction::ShowHelp => app.set_mode(AppMode::Help),
                            AppAction::ToggleSelection => app.toggle_selection(),
                            AppAction::MoveUp => app.move_up(),
                            AppAction::MoveDown => app.move_down(),
                            AppAction::PageUp => app.page_up(),
                            AppAction::PageDown => app.page_down(),
                            AppAction::MoveToTop => app.move_to_top(),
                            AppAction::MoveToBottom => app.move_to_bottom(),
                            AppAction::SearchChar(c) => app.add_search_char(c),
                            AppAction::SearchBackspace => app.search_backspace(),
                        }
                    }
                }
                AppEvent::Tick => {
                    // Handle periodic updates if needed
                }
                AppEvent::Quit => app.quit(),
            }
        }
    }

    Ok(())
}

async fn run_non_interactive_mode(cli: &Cli) -> Result<()> {
    let traverser = DirectoryTraverser::new(cli.respect_gitignore, cli.max_file_size, cli.include_all);
    let tree = traverser.traverse(&cli.root)?;

    generate_output(&tree, cli)?;
    Ok(())
}

async fn run_direct_mode(cli: &Cli, _include: &[String], _exclude: &[String]) -> Result<()> {
    let traverser = DirectoryTraverser::new(cli.respect_gitignore, cli.max_file_size, false);
    let tree = traverser.traverse(&cli.root)?;

    // TODO: Apply include/exclude patterns to the tree
    generate_output(&tree, cli)?;
    Ok(())
}

fn generate_output(tree: &directory::tree::DirectoryTree, cli: &Cli) -> Result<()> {
    let formatter = OutputFormatter::new()
        .with_metadata(false)
        .with_line_numbers(false);

    let writer = OutputWriter::new().with_formatter(formatter);

    if let Some(output_path) = &cli.output {
        writer.write_to_file(tree, output_path)?;
        println!("✓ Output written to: {}", output_path.display());
    } else {
        writer.write_to_clipboard_or_prompt(tree)?;
    }

    Ok(())
}