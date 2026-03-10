pub mod cli;
pub mod config;
pub mod constants;
pub mod directory;
pub mod fuzzy;
pub mod output;
pub mod ui;

use anyhow::Result;
use cli::Cli;
use config::settings::Settings;
use constants::DEFAULT_MAX_FILE_SIZE;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use directory::state::SelectionState;
use directory::traversal::DirectoryTraverser;
use directory::tree::is_text_file;
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use output::formatter::OutputFormatter;
use output::writer::OutputWriter;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::io;
use std::time::Duration;
use ui::app::{App, AppMode, ScanState};
use ui::events::{AppAction, AppEvent, EventHandler, handle_key_event};
use ui::interface::draw_ui;

pub async fn run_interactive_mode(cli: &Cli, settings: &Settings) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create traverser and start background scan
    let max_file_size = if cli.max_file_size == constants::DEFAULT_MAX_FILE_SIZE {
        settings.max_file_size
    } else {
        cli.max_file_size
    };
    let respect_gitignore = cli.respect_gitignore.unwrap_or(settings.respect_gitignore);
    let show_hidden = cli.show_hidden.unwrap_or(settings.show_hidden);
    let initial_state = if cli.include_all {
        SelectionState::Included
    } else {
        SelectionState::Excluded
    };

    let (extra_text_ext, exclude_text_ext) = settings.text_extension_overrides();
    let traverser = DirectoryTraverser::new(
        respect_gitignore,
        show_hidden,
        max_file_size,
        cli.include_all,
        extra_text_ext,
        exclude_text_ext,
    );
    let (rx, handle) = traverser.traverse_streaming(&cli.root);

    let mut app = App::new_streaming(cli.root.clone(), rx, handle, initial_state);

    let event_handler = EventHandler::new();
    let result = run_app(&mut terminal, &mut app, &event_handler, cli, settings).await;

    // Cleanup: drop receiver so scanner thread will exit on next send
    app.scan_receiver.take();
    // Don't join the scanner thread — it may still be walking a large directory.
    // Dropping the receiver is sufficient; the thread will exit on its next send attempt.
    drop(app.scan_handle.take());

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
    let mut patterns_applied = false;
    loop {
        if app.should_quit {
            break;
        }

        app.drain_scan_entries();

        if !patterns_applied && app.scan_state == ScanState::Complete {
            if !cli.include.is_empty() || !cli.exclude.is_empty() {
                apply_patterns(&mut app.tree, &cli.include, &cli.exclude);
                app.update_filtered_results();
            }

            if !cli.paths.is_empty() {
                let warnings = apply_explicit_paths(&mut app.tree, &cli.root, &cli.paths);
                app.update_filtered_results();
                if !warnings.is_empty() {
                    app.set_notification(warnings.join("; "));
                }
            }

            patterns_applied = true;
        }

        terminal.draw(|f| draw_ui(f, app))?;

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
                            AppAction::HalfPageUp => app.half_page_up(),
                            AppAction::HalfPageDown => app.half_page_down(),
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
                AppEvent::Tick => {}
                AppEvent::Quit => app.quit(),
            }
        }
    }

    Ok(())
}

pub async fn run_direct_mode(cli: &Cli, settings: &Settings) -> Result<()> {
    let tree = build_directory_tree(cli, settings)?;
    handle_output(&tree, cli, settings, false)?;
    Ok(())
}

/// Build the directory tree with common logic for both modes
pub fn build_directory_tree(
    cli: &Cli,
    settings: &Settings,
) -> Result<directory::tree::DirectoryTree> {
    let max_file_size = if cli.max_file_size == DEFAULT_MAX_FILE_SIZE {
        settings.max_file_size
    } else {
        cli.max_file_size
    };
    let respect_gitignore = cli.respect_gitignore.unwrap_or(settings.respect_gitignore);
    let show_hidden = cli.show_hidden.unwrap_or(settings.show_hidden);

    let (extra_text_ext, exclude_text_ext) = settings.text_extension_overrides();

    // Fast path: only -p, no -i/-e/-I
    if cli.include.is_empty() && cli.exclude.is_empty() && !cli.paths.is_empty() && !cli.include_all {
        return Ok(build_tree_from_paths(
            &cli.root,
            &cli.paths,
            max_file_size,
            respect_gitignore,
            show_hidden,
            &extra_text_ext,
            &exclude_text_ext,
        ));
    }

    let traverser = DirectoryTraverser::new(
        respect_gitignore,
        show_hidden,
        max_file_size,
        cli.include_all,
        extra_text_ext.clone(),
        exclude_text_ext.clone(),
    );

    let overrides = if !cli.include.is_empty() || !cli.exclude.is_empty() {
        let mut ob = OverrideBuilder::new(&cli.root);
        for pattern in &cli.include {
            ob.add(pattern)?;
        }
        for pattern in &cli.exclude {
            ob.add(&format!("!{}", pattern))?;
        }
        Some(ob.build()?)
    } else {
        None
    };

    let mut tree = traverser.traverse(&cli.root, overrides)?;

    // Mixed: graft -p paths into tree after walker
    if !cli.paths.is_empty() {
        graft_paths_into_tree(&mut tree, &cli.root, &cli.paths, max_file_size, respect_gitignore, show_hidden, &extra_text_ext, &exclude_text_ext);
    }

    Ok(tree)
}

fn resolve_path(root: &Path, path_arg: &Path) -> PathBuf {
    if path_arg.is_absolute() {
        path_arg.to_path_buf()
    } else {
        root.join(path_arg)
    }
}

/// Ensure all ancestor directories between root and the given path exist in the tree.
/// Returns the index of the immediate parent directory, or None if path is not under root.
fn ensure_ancestors(
    tree: &mut directory::tree::DirectoryTree,
    root: &Path,
    path: &Path,
    extra_text_ext: &HashSet<String>,
    exclude_text_ext: &HashSet<String>,
) -> Option<usize> {
    let relative = path.strip_prefix(root).ok()?;
    let parent_relative = relative.parent()?;

    let mut current = root.to_path_buf();
    for component in parent_relative.components() {
        let next = current.join(component);
        if !tree.path_to_index.contains_key(&next) {
            tree.add_node(next.clone(), true, &current, extra_text_ext, exclude_text_ext);
            if let Some(&idx) = tree.path_to_index.get(&next) {
                tree.set_state(idx, SelectionState::Included);
            }
        }
        current = next;
    }

    tree.path_to_index.get(&current).copied()
}

fn add_file_to_tree(
    tree: &mut directory::tree::DirectoryTree,
    root: &Path,
    resolved: &Path,
    max_file_size: u64,
    extra_text_ext: &HashSet<String>,
    exclude_text_ext: &HashSet<String>,
) {
    if let Ok(metadata) = std::fs::metadata(resolved) {
        if metadata.len() > max_file_size {
            eprintln!("warning: file exceeds size limit, skipping: {}", resolved.display());
            return;
        }

        ensure_ancestors(tree, root, resolved, extra_text_ext, exclude_text_ext);
        let parent_path = resolved.parent().unwrap_or(root);
        if let Some(node_index) = tree.add_node(resolved.to_path_buf(), false, parent_path, extra_text_ext, exclude_text_ext) {
            if let Some(node) = tree.get_node_mut(node_index) {
                node.size = Some(metadata.len());
                node.is_text_file = is_text_file(resolved, extra_text_ext, exclude_text_ext);
            }
            tree.set_state(node_index, SelectionState::Included);
        }
    }
}

fn build_tree_from_paths(
    root: &Path,
    paths: &[PathBuf],
    max_file_size: u64,
    respect_gitignore: bool,
    show_hidden: bool,
    extra_text_ext: &HashSet<String>,
    exclude_text_ext: &HashSet<String>,
) -> directory::tree::DirectoryTree {
    let canonical_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let mut tree = directory::tree::DirectoryTree::new(canonical_root.clone());
    tree.set_state(tree.root_index, SelectionState::Included);

    for path_arg in paths {
        let resolved = resolve_path(&canonical_root, path_arg);
        let resolved = std::fs::canonicalize(&resolved).unwrap_or(resolved);

        if !resolved.exists() {
            eprintln!("warning: path not found: {}", path_arg.display());
            continue;
        }
        if !resolved.starts_with(&canonical_root) {
            eprintln!("warning: path outside root, skipping: {}", path_arg.display());
            continue;
        }

        if resolved.is_file() {
            add_file_to_tree(&mut tree, &canonical_root, &resolved, max_file_size, extra_text_ext, exclude_text_ext);
        } else if resolved.is_dir() {
            // Ensure ancestor dirs exist
            ensure_ancestors(&mut tree, &canonical_root, &resolved, extra_text_ext, exclude_text_ext);
            let dir_parent = resolved.parent().unwrap_or(&canonical_root);
            if !tree.path_to_index.contains_key(&resolved) {
                tree.add_node(resolved.clone(), true, dir_parent, extra_text_ext, exclude_text_ext);
            }
            if let Some(&idx) = tree.path_to_index.get(&resolved) {
                tree.set_state(idx, SelectionState::Included);
            }

            let mut builder = WalkBuilder::new(&resolved);
            builder.hidden(!show_hidden);
            if !respect_gitignore {
                builder.git_ignore(false).git_global(false).git_exclude(false);
            }

            for entry in builder.build() {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path = entry.path();
                if path == resolved {
                    continue;
                }

                let is_directory = entry.file_type().map_or(false, |ft| ft.is_dir());
                let entry_parent = path.parent().unwrap_or(&resolved);

                // Ensure parent dirs in the tree
                if !tree.path_to_index.contains_key(entry_parent) {
                    ensure_ancestors(&mut tree, &canonical_root, path, extra_text_ext, exclude_text_ext);
                }

                if is_directory {
                    if let Some(idx) = tree.add_node(path.to_path_buf(), true, entry_parent, extra_text_ext, exclude_text_ext) {
                        tree.set_state(idx, SelectionState::Included);
                    }
                } else {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        if metadata.len() > max_file_size {
                            continue;
                        }
                        if let Some(idx) = tree.add_node(path.to_path_buf(), false, entry_parent, extra_text_ext, exclude_text_ext) {
                            if let Some(node) = tree.get_node_mut(idx) {
                                node.size = Some(metadata.len());
                                node.is_text_file = is_text_file(path, extra_text_ext, exclude_text_ext);
                            }
                            tree.set_state(idx, SelectionState::Included);
                        }
                    }
                }
            }
        }
    }
    tree
}

fn graft_paths_into_tree(
    tree: &mut directory::tree::DirectoryTree,
    root: &Path,
    paths: &[PathBuf],
    max_file_size: u64,
    respect_gitignore: bool,
    show_hidden: bool,
    extra_text_ext: &HashSet<String>,
    exclude_text_ext: &HashSet<String>,
) {
    let canonical_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());

    for path_arg in paths {
        let resolved = resolve_path(&canonical_root, path_arg);
        let resolved = std::fs::canonicalize(&resolved).unwrap_or(resolved);

        if !resolved.exists() {
            eprintln!("warning: path not found: {}", path_arg.display());
            continue;
        }
        if !resolved.starts_with(&canonical_root) {
            eprintln!("warning: path outside root, skipping: {}", path_arg.display());
            continue;
        }

        if resolved.is_file() {
            // If already in tree, just set it to Included
            if let Some(&idx) = tree.path_to_index.get(&resolved) {
                tree.set_state(idx, SelectionState::Included);
            } else {
                add_file_to_tree(tree, &canonical_root, &resolved, max_file_size, extra_text_ext, exclude_text_ext);
            }
        } else if resolved.is_dir() {
            // If directory already exists in tree, include it and all children
            if let Some(&idx) = tree.path_to_index.get(&resolved) {
                tree.set_state(idx, SelectionState::Included);
            } else {
                // Walk and add
                ensure_ancestors(tree, &canonical_root, &resolved, extra_text_ext, exclude_text_ext);
                let dir_parent = resolved.parent().unwrap_or(&canonical_root);
                if let Some(idx) = tree.add_node(resolved.clone(), true, dir_parent, extra_text_ext, exclude_text_ext) {
                    tree.set_state(idx, SelectionState::Included);
                }

                let mut builder = WalkBuilder::new(&resolved);
                builder.hidden(!show_hidden);
                if !respect_gitignore {
                    builder.git_ignore(false).git_global(false).git_exclude(false);
                }

                for entry in builder.build() {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    let path = entry.path();
                    if path == resolved {
                        continue;
                    }

                    let is_directory = entry.file_type().map_or(false, |ft| ft.is_dir());
                    let entry_parent = path.parent().unwrap_or(&resolved);

                    if !tree.path_to_index.contains_key(entry_parent) {
                        ensure_ancestors(tree, &canonical_root, path, extra_text_ext, exclude_text_ext);
                    }

                    if is_directory {
                        if let Some(idx) = tree.add_node(path.to_path_buf(), true, entry_parent, extra_text_ext, exclude_text_ext) {
                            tree.set_state(idx, SelectionState::Included);
                        }
                    } else {
                        if let Ok(metadata) = std::fs::metadata(path) {
                            if metadata.len() > max_file_size {
                                continue;
                            }
                            if let Some(idx) = tree.add_node(path.to_path_buf(), false, entry_parent, extra_text_ext, exclude_text_ext) {
                                if let Some(node) = tree.get_node_mut(idx) {
                                    node.size = Some(metadata.len());
                                    node.is_text_file = is_text_file(path, extra_text_ext, exclude_text_ext);
                                }
                                tree.set_state(idx, SelectionState::Included);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn apply_explicit_paths(
    tree: &mut directory::tree::DirectoryTree,
    root: &Path,
    paths: &[PathBuf],
) -> Vec<String> {
    let canonical_root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let mut warnings = Vec::new();

    for path_arg in paths {
        let resolved = resolve_path(&canonical_root, path_arg);
        let resolved = std::fs::canonicalize(&resolved).unwrap_or(resolved);

        if !resolved.exists() {
            warnings.push(format!("path not found: {}", path_arg.display()));
            continue;
        }
        if !resolved.starts_with(&canonical_root) {
            warnings.push(format!("path outside root: {}", path_arg.display()));
            continue;
        }

        if let Some(&idx) = tree.path_to_index.get(&resolved) {
            tree.set_state(idx, SelectionState::Included);
        } else {
            warnings.push(format!("path not in tree: {}", path_arg.display()));
        }
    }

    warnings
}

fn apply_patterns(
    tree: &mut directory::tree::DirectoryTree,
    include: &[String],
    exclude: &[String],
) {
    use directory::state::SelectionState;
    use globset::Glob;

    let include_all = include.is_empty();

    let include_matchers: Vec<_> = include
        .iter()
        .filter_map(|p| Glob::new(p).ok().map(|g| g.compile_matcher()))
        .collect();
    let exclude_matchers: Vec<_> = exclude
        .iter()
        .filter_map(|p| Glob::new(p).ok().map(|g| g.compile_matcher()))
        .collect();

    for i in 0..tree.nodes.len() {
        if let Some(node) = tree.nodes.get(i) {
            let relative_path = if let Some(root_node) = tree.nodes.get(tree.root_index) {
                node.path
                    .strip_prefix(&root_node.path)
                    .unwrap_or(&node.path)
                    .to_string_lossy()
            } else {
                node.path.to_string_lossy()
            };

            let mut should_include = include_all;

            for matcher in &include_matchers {
                if matcher.is_match(relative_path.as_ref())
                    || matcher.is_match(&node.name)
                {
                    should_include = true;
                    break;
                }
            }

            for matcher in &exclude_matchers {
                if matcher.is_match(relative_path.as_ref())
                    || matcher.is_match(&node.name)
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

pub enum OutputAction {
    Quit,
    StartFileSave(String),
    Continue,
}

/// Unified output handler for both interactive and direct modes
pub fn handle_output(
    tree: &directory::tree::DirectoryTree,
    cli: &Cli,
    settings: &Settings,
    is_interactive: bool,
) -> Result<OutputAction> {
    let formatter = OutputFormatter::new()
        .with_line_numbers(false);
    let content = formatter.format_output(tree)?;

    if content.trim().is_empty() {
        println!("⚠ No content included. Please include at least one file.");
        return Ok(OutputAction::Quit);
    }

    if let Some(output_path) = &cli.output {
        let writer = OutputWriter::new().with_formatter(formatter);
        writer.write_to_file(tree, output_path)?;
        println!("✓ Output written to: {}", output_path.display());
        return Ok(OutputAction::Quit);
    }

    if content.len() <= settings.max_clipboard_size {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(&content).is_ok() {
                println!("✓ Output copied to clipboard ({} bytes)", content.len());
                return Ok(OutputAction::Quit);
            }
        }
    }

    if is_interactive {
        Ok(OutputAction::StartFileSave(content))
    } else {
        save_file_with_text_prompt(tree, &content, settings)?;
        Ok(OutputAction::Continue)
    }
}

pub fn handle_export(app: &mut App, cli: &Cli, settings: &Settings) -> Result<()> {
    match handle_output(&app.tree, cli, settings, true)? {
        OutputAction::Quit => app.quit(),
        OutputAction::StartFileSave(content) => app.start_file_save(content),
        OutputAction::Continue => {}
    }
    Ok(())
}

pub fn save_file_with_text_prompt(
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
        if !input.contains('.') {
            format!("{}.md", input)
        } else {
            input.to_string()
        }
    };

    let path = Path::new(&filename);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    println!("✓ Output saved to: {}", path.display());
    Ok(())
}

pub fn save_file_from_dialog(app: &App, content: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;

    let filename = if app.file_save_input.trim().is_empty() {
        OutputWriter::generate_default_filename(&app.tree)
    } else {
        let input = app.file_save_input.trim();
        if !input.contains('.') {
            format!("{}.md", input)
        } else {
            input.to_string()
        }
    };

    let path = Path::new(&filename);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, content)?;
    println!("✓ Output saved to: {}", path.display());
    Ok(())
}
