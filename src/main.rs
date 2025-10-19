mod cli;
mod config;
mod constants;
mod directory;
mod fuzzy;
mod output;
mod ui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use config::settings::Settings;
use constants::DEFAULT_MAX_FILE_SIZE;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use directory::traversal::DirectoryTraverser;
use output::formatter::OutputFormatter;
use output::writer::OutputWriter;
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::io;
use std::time::Duration;
use ui::app::{App, AppMode};
use ui::events::{AppAction, AppEvent, EventHandler, handle_key_event};
use ui::interface::draw_ui;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let settings = Settings::load_with_project_root(&cli.root);

    match cli.command.as_ref().unwrap_or(&Commands::Interactive) {
        Commands::Interactive => {
            run_interactive_mode(&cli, &settings).await?;
        }
        Commands::Direct => {
            run_direct_mode(&cli, &settings).await?;
        }
    }

    Ok(())
}

async fn run_interactive_mode(cli: &Cli, settings: &Settings) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create application state
    let tree = build_directory_tree(cli, settings)?;
    let mut app = App::new(tree);

    let event_handler = EventHandler::new();
    let result = run_app(&mut terminal, &mut app, &event_handler, cli, settings).await;

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
    settings: &Settings,
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

                    if let Some(action) = handle_key_event(key_event, &app.mode) {
                        match action {
                            AppAction::Escape => app.handle_escape(),
                            AppAction::Export => {
                                handle_export(app, cli, settings)?;
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
                            AppAction::FileSaveChar(c) => app.add_file_save_char(c),
                            AppAction::FileSaveBackspace => app.file_save_backspace(),
                            AppAction::FileSaveConfirm => {
                                if let Some(content) = &app.pending_content.clone() {
                                    save_file_from_dialog(&app, content)?;
                                    app.quit();
                                }
                            }
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

async fn run_direct_mode(cli: &Cli, settings: &Settings) -> Result<()> {
    let tree = build_directory_tree(cli, settings)?;
    handle_output(&tree, cli, settings, false)?;
    Ok(())
}

/// Build the directory tree with common logic for both modes
fn build_directory_tree(cli: &Cli, settings: &Settings) -> Result<directory::tree::DirectoryTree> {
    let max_file_size = if cli.max_file_size == DEFAULT_MAX_FILE_SIZE {
        // If using default CLI value
        settings.max_file_size // Use config file value
    } else {
        cli.max_file_size // Use explicitly set CLI value
    };
    let respect_gitignore = cli.respect_gitignore.unwrap_or(settings.respect_gitignore);
    let show_hidden = cli.show_hidden.unwrap_or(settings.show_hidden);
    let traverser = DirectoryTraverser::new(
        respect_gitignore,
        show_hidden,
        max_file_size,
        cli.include_all,
    );
    let mut tree = traverser.traverse(&cli.root)?;

    // Apply include/exclude patterns if provided
    if !cli.include.is_empty() || !cli.exclude.is_empty() {
        apply_patterns(&mut tree, &cli.include, &cli.exclude);
    }

    Ok(tree)
}

fn apply_patterns(
    tree: &mut directory::tree::DirectoryTree,
    include: &[String],
    exclude: &[String],
) {
    use directory::state::SelectionState;

    // If no include patterns are specified, include everything by default
    let include_all = include.is_empty();

    for i in 0..tree.nodes.len() {
        if let Some(node) = tree.nodes.get(i) {
            // Use relative path from the root for pattern matching
            let relative_path = if let Some(root_node) = tree.nodes.get(tree.root_index) {
                node.path
                    .strip_prefix(&root_node.path)
                    .unwrap_or(&node.path)
                    .to_string_lossy()
            } else {
                node.path.to_string_lossy()
            };

            let mut should_include = include_all;

            // Check include patterns
            for pattern in include {
                if path_matches_pattern(&relative_path, pattern)
                    || path_matches_pattern(&node.name, pattern)
                {
                    should_include = true;
                    break;
                }
            }

            // Check exclude patterns (these override includes)
            for pattern in exclude {
                if path_matches_pattern(&relative_path, pattern)
                    || path_matches_pattern(&node.name, pattern)
                {
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

enum OutputAction {
    Quit,
    StartFileSave(String),
    Continue,
}

/// Unified output handler for both interactive and direct modes
///
/// Returns OutputAction to indicate what the caller should do
fn handle_output(
    tree: &directory::tree::DirectoryTree,
    cli: &Cli,
    settings: &Settings,
    is_interactive: bool,
) -> Result<OutputAction> {
    // No output file specified, format the content
    let formatter = OutputFormatter::new()
        .with_metadata(false)
        .with_line_numbers(false);
    let content = formatter.format_output(tree)?;

    // Check if content is empty (no files included)
    if content.trim().is_empty() {
        println!("⚠ No content included. Please include at least one file.");
        return Ok(OutputAction::Quit);
    }

    // If -o flag is provided, write directly to file
    if let Some(output_path) = &cli.output {
        let writer = OutputWriter::new().with_formatter(formatter);
        writer.write_to_file(tree, output_path)?;
        println!("✓ Output written to: {}", output_path.display());
        return Ok(OutputAction::Quit);
    }

    // Try clipboard if content is small enough
    if content.len() <= settings.max_clipboard_size {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(&content).is_ok() {
                println!("✓ Output copied to clipboard ({} bytes)", content.len());
                return Ok(OutputAction::Quit);
            }
        }
    }

    // Clipboard failed or content too large
    if is_interactive {
        // Interactive mode: start file save dialog
        Ok(OutputAction::StartFileSave(content))
    } else {
        // Direct mode: use text prompt
        save_file_with_text_prompt(tree, &content, settings)?;
        Ok(OutputAction::Continue)
    }
}

fn handle_export(app: &mut App, cli: &Cli, settings: &Settings) -> Result<()> {
    match handle_output(&app.tree, cli, settings, true)? {
        OutputAction::Quit => app.quit(),
        OutputAction::StartFileSave(content) => app.start_file_save(content),
        OutputAction::Continue => {}
    }
    Ok(())
}

fn save_file_with_text_prompt(
    tree: &directory::tree::DirectoryTree,
    content: &str,
    settings: &Settings,
) -> Result<()> {
    use std::fs;
    use std::io::{self, Write};
    use std::path::Path;

    if content.len() > settings.max_clipboard_size {
        println!(
            "⚠ Output is too large for clipboard ({} bytes > {})",
            content.len(),
            settings.format_clipboard_size()
        );
    }

    print!("Enter file path to save output (or press Enter for default): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    let filename = if input.is_empty() {
        OutputWriter::generate_default_filename(tree)
    } else {
        // Add .md extension if not present and doesn't have any extension
        if !input.contains('.') {
            format!("{}.md", input)
        } else {
            input.to_string()
        }
    };

    let path = Path::new(&filename);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    println!("✓ Output saved to: {}", path.display());
    Ok(())
}

fn save_file_from_dialog(app: &App, content: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;

    let filename = if app.file_save_input.trim().is_empty() {
        // Generate default filename
        OutputWriter::generate_default_filename(&app.tree)
    } else {
        let input = app.file_save_input.trim();
        // Add .md extension if not present and doesn't have any extension
        if !input.contains('.') {
            format!("{}.md", input)
        } else {
            input.to_string()
        }
    };

    let path = Path::new(&filename);

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    println!("✓ Output saved to: {}", path.display());
    Ok(())
}
