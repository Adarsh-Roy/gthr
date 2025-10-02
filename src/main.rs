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
            run_interactive_mode(&cli).await?;
        }
        Commands::Direct => {
            run_direct_mode(&cli).await?;
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
    let mut tree = traverser.traverse(&cli.root)?;

    // Apply include/exclude patterns if provided
    if !cli.include.is_empty() || !cli.exclude.is_empty() {
        apply_patterns(&mut tree, &cli.include, &cli.exclude);
    }

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
                            AppAction::Export => {
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


async fn run_direct_mode(cli: &Cli) -> Result<()> {
    let traverser = DirectoryTraverser::new(cli.respect_gitignore, cli.max_file_size, cli.include_all);
    let mut tree = traverser.traverse(&cli.root)?;

    // Apply include/exclude patterns to the tree
    apply_patterns(&mut tree, &cli.include, &cli.exclude);

    generate_output(&tree, cli)?;
    Ok(())
}

fn apply_patterns(tree: &mut directory::tree::DirectoryTree, include: &[String], exclude: &[String]) {
    use directory::state::SelectionState;

    // If no include patterns are specified, include everything by default
    let include_all = include.is_empty();

    for i in 0..tree.nodes.len() {
        if let Some(node) = tree.nodes.get(i) {
            // Use relative path from the root for pattern matching
            let relative_path = if let Some(root_node) = tree.nodes.get(tree.root_index) {
                node.path.strip_prefix(&root_node.path).unwrap_or(&node.path).to_string_lossy()
            } else {
                node.path.to_string_lossy()
            };

            let mut should_include = include_all;

            // Check include patterns
            for pattern in include {
                if path_matches_pattern(&relative_path, pattern) || path_matches_pattern(&node.name, pattern) {
                    should_include = true;
                    break;
                }
            }

            // Check exclude patterns (these override includes)
            for pattern in exclude {
                if path_matches_pattern(&relative_path, pattern) || path_matches_pattern(&node.name, pattern) {
                    should_include = false;
                    break;
                }
            }

            let new_state = if should_include {
                SelectionState::Included
            } else {
                SelectionState::Excluded
            };

            tree.set_state(i, new_state);
        }
    }
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    // Simple glob-like matching
    if pattern == "**/*" {
        return true;
    }

    // Handle common patterns
    if pattern.ends_with("*") {
        let prefix = &pattern[..pattern.len() - 1];
        return path.starts_with(prefix);
    }

    if pattern.starts_with("*") {
        let suffix = &pattern[1..];
        return path.ends_with(suffix);
    }

    // Convert glob pattern to regex-like matching
    let regex_pattern = pattern
        .replace(".", "\\.")
        .replace("**", ".*")
        .replace("*", "[^/]*")
        .replace("?", ".");

    if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
        regex.is_match(path)
    } else {
        // Fallback to simple equality check
        path == pattern
    }
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