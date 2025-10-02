use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

use crate::directory::state::SelectionState;
use crate::fuzzy::filter::get_node_display_path;
use crate::ui::app::{App, AppMode};

pub fn draw_ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    match app.mode {
        AppMode::Main => draw_main_interface(f, app, size),
        AppMode::Help => draw_help_interface(f, app, size),
        AppMode::FileSave => draw_file_save_dialog(f, app, size),
    }
}

fn draw_main_interface(f: &mut Frame, app: &mut App, area: Rect) {
    // Clear the background for transparency
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(0),    // File list
            Constraint::Length(3), // Status bar
        ])
        .split(area);

    draw_search_bar(f, app, chunks[0]);
    draw_file_list(f, app, chunks[1]);
    draw_status_bar(f, app, chunks[2]);
}

fn draw_search_bar(f: &mut Frame, app: &App, area: Rect) {
    let search_text = if app.search_query.is_empty() {
        "Type to search files and directories..."
    } else {
        &app.search_query
    };

    let style = if app.search_query.is_empty() {
        app.color_scheme.help_text
    } else {
        app.color_scheme.text
    };

    let search_paragraph = Paragraph::new(search_text)
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("🔍 Search")
                .border_style(app.color_scheme.border),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(search_paragraph, area);
}

fn draw_file_list(f: &mut Frame, app: &mut App, area: Rect) {
    // Calculate the actual viewport height for the file list area
    // Subtract 2 for the borders
    let actual_viewport_height = area.height.saturating_sub(2) as usize;

    // Update the app's viewport height to match the actual visible area
    app.viewport_height = actual_viewport_height;

    let items: Vec<ListItem> = app
        .filtered_results
        .visible_items
        .iter()
        .skip(app.scroll_offset)
        .take(actual_viewport_height)
        .enumerate()
        .map(|(viewport_index, &tree_index)| {
            // viewport_index is now 0-based index within the visible viewport
            // The actual index in the filtered results is scroll_offset + viewport_index
            let actual_index = app.scroll_offset + viewport_index;
            create_list_item(app, tree_index, actual_index == app.selected_index)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Files and Directories (Enter = toggle ✓/✗)")
                .border_style(app.color_scheme.border),
        )
        .style(app.color_scheme.background);

    f.render_widget(list, area);
}

fn create_list_item(app: &App, tree_index: usize, is_selected: bool) -> ListItem {
    if let Some(node) = app.tree.get_node(tree_index) {
        let display_path = get_node_display_path(&app.tree, tree_index);

        let state_indicator = match node.state {
            SelectionState::Included => "✓",
            SelectionState::Excluded => "✗",
            SelectionState::Partial => "◐",
        };

        let file_type_indicator = if node.is_directory { "📁" } else { "📄" };

        let cursor_indicator = if is_selected { "▶ " } else { "  " };

        // Get base style for the state, not influenced by selection
        let base_style = app.color_scheme.get_state_style(node.state);

        let spans = vec![
            Span::styled(cursor_indicator, app.color_scheme.text),
            Span::styled(format!("{} ", state_indicator), base_style),
            Span::styled(format!("{} ", file_type_indicator), app.color_scheme.text),
            Span::styled(display_path, base_style),
        ];

        if let Some(size) = node.size {
            let size_str = format_file_size(size);
            let line = Line::from(spans);

            // Add size information for files
            let mut full_spans = line.spans;
            full_spans.push(Span::styled(
                format!(" ({})", size_str),
                app.color_scheme.help_text,
            ));

            ListItem::new(Line::from(full_spans))
        } else {
            ListItem::new(Line::from(spans))
        }
    } else {
        ListItem::new("Invalid node")
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.get_stats();

    let left_text = format!(
        "Files: {}/{} | Size: {} | Filtered: {}",
        stats.included_files,
        stats.total_files,
        stats.format_size(),
        stats.filtered_count
    );

    // Adjust help text based on available width
    let available_width = area.width.saturating_sub(4) as usize; // Account for borders
    let left_text_len = left_text.len();
    let remaining_width = available_width.saturating_sub(left_text_len);

    let right_text = if remaining_width > 80 {
        "↑/↓: Move | Enter: Toggle ✓/✗ | Ctrl+E: Export | Ctrl+H: Help"
    } else if remaining_width > 60 {
        "↑/↓: Move | Enter: Toggle | Ctrl+E: Export | Ctrl+H: Help"
    } else if remaining_width > 40 {
        "↑/↓: Move | Ctrl+E: Export | Ctrl+H: Help"
    } else if remaining_width > 25 {
        "↑/↓: Move | Ctrl+E: Export"
    } else {
        "Ctrl+E: Export"
    };

    let status_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_paragraph = Paragraph::new(left_text)
        .style(app.color_scheme.text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.color_scheme.border),
        );

    let right_paragraph = Paragraph::new(right_text)
        .style(app.color_scheme.help_text)
        .alignment(Alignment::Right)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.color_scheme.border),
        );

    f.render_widget(left_paragraph, status_chunks[0]);
    f.render_widget(right_paragraph, status_chunks[1]);
}

fn draw_help_interface(f: &mut Frame, app: &App, area: Rect) {
    let help_text = vec![
        Line::from("gthr - Help"),
        Line::from(""),
        Line::from("Search:"),
        Line::from("  Type       Add any character to search (letters, numbers, symbols)"),
        Line::from("  Backspace  Delete search character"),
        Line::from("  Esc        Clear search text (or quit if empty)"),
        Line::from(""),
        Line::from("Navigation:"),
        Line::from("  ↑/↓        Move up/down"),
        Line::from("  ←/→        Move up/down (alternative)"),
        Line::from(""),
        Line::from("Selection:"),
        Line::from("  Enter      Toggle ✓ included / ✗ excluded"),
        Line::from(""),
        Line::from("Actions:"),
        Line::from("  Ctrl+E     Export output and quit"),
        Line::from("  Ctrl+H     Show this help"),
        Line::from("  Esc        Clear search (or quit if search empty)"),
        Line::from(""),
        Line::from("Colors:"),
        Line::from(vec![
            Span::styled("  ✓ ", app.color_scheme.included),
            Span::from("Included"),
        ]),
        Line::from(vec![
            Span::styled("  ✗ ", app.color_scheme.excluded),
            Span::from("Excluded"),
        ]),
        Line::from(vec![
            Span::styled("  ◐ ", app.color_scheme.partial),
            Span::from("Partially included"),
        ]),
        Line::from(""),
        Line::from("Press any key to return..."),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .style(app.color_scheme.text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Help")
                .border_style(app.color_scheme.border),
        )
        .wrap(Wrap { trim: true });

    // Center the help dialog
    let popup_area = centered_rect(80, 90, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(help_paragraph, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn draw_file_save_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered popup
    let popup_area = centered_rect(60, 25, area);

    // Clear the popup area
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Save File")
        .borders(Borders::ALL)
        .border_style(app.color_scheme.border)
        .style(app.color_scheme.background);

    // Split the popup area for title, input, and instructions
    let popup_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Instructions
            Constraint::Length(3), // Input field
            Constraint::Length(1), // Help text
        ])
        .split(popup_area);

    // Content size info
    let content_size = if let Some(content) = &app.pending_content {
        format_file_size(content.len() as u64)
    } else {
        "Unknown".to_string()
    };

    let instructions = Paragraph::new(format!(
        "Output is too large for clipboard ({}). Enter file path to save:",
        content_size
    ))
    .style(app.color_scheme.text)
    .wrap(Wrap { trim: true });

    // Input field
    let input_text = if app.file_save_input.is_empty() {
        "📁 Enter file path (or press Enter for default)".to_string()
    } else {
        app.file_save_input.clone()
    };

    let input = Paragraph::new(input_text)
        .style(if app.file_save_input.is_empty() {
            app.color_scheme.help_text
        } else {
            app.color_scheme.text
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(app.color_scheme.border)
                .title("File Path"),
        );

    let help_text = Paragraph::new("Enter: Save | Esc: Cancel")
        .style(app.color_scheme.help_text)
        .alignment(Alignment::Center);

    // Render all components
    f.render_widget(block, popup_area);
    f.render_widget(instructions, popup_chunks[0]);
    f.render_widget(input, popup_chunks[1]);
    f.render_widget(help_text, popup_chunks[2]);

    // Position cursor in the input field
    if !app.file_save_input.is_empty() {
        f.set_cursor(
            popup_chunks[1].x + app.file_save_input.len() as u16 + 1,
            popup_chunks[1].y + 1,
        );
    }
}

fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size_f = size as f64;
    let mut unit_index = 0;

    while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size_f, UNITS[unit_index])
    }
}
