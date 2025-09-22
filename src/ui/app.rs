use crate::directory::tree::DirectoryTree;
use crate::directory::state::SelectionState;
use crate::fuzzy::filter::{filter_tree_nodes, FilteredResults};
use crate::ui::colors::ColorScheme;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Main,
    Help,
}

pub struct App {
    pub tree: DirectoryTree,
    pub filtered_results: FilteredResults,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub search_query: String,
    pub mode: AppMode,
    pub color_scheme: ColorScheme,
    pub should_quit: bool,
    pub viewport_height: usize,
}

impl App {
    pub fn new(tree: DirectoryTree) -> Self {
        let mut app = Self {
            filtered_results: FilteredResults::new(),
            tree,
            selected_index: 0,
            scroll_offset: 0,
            search_query: String::new(),
            mode: AppMode::Main,
            color_scheme: ColorScheme::default(),
            should_quit: false,
            viewport_height: 20, // Default, will be updated by UI
        };

        app.update_filtered_results();
        app
    }

    pub fn update_filtered_results(&mut self) {
        self.filtered_results = filter_tree_nodes(&self.tree, &self.search_query);

        // Adjust selected index if it's out of bounds
        if self.selected_index >= self.filtered_results.len() && !self.filtered_results.is_empty() {
            self.selected_index = self.filtered_results.len() - 1;
        }

        self.update_scroll();
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll();
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.filtered_results.len() {
            self.selected_index += 1;
            self.update_scroll();
        }
    }

    pub fn page_up(&mut self) {
        let page_size = self.viewport_height.saturating_sub(1);
        self.selected_index = self.selected_index.saturating_sub(page_size);
        self.update_scroll();
    }

    pub fn page_down(&mut self) {
        let page_size = self.viewport_height.saturating_sub(1);
        self.selected_index = (self.selected_index + page_size).min(self.filtered_results.len().saturating_sub(1));
        self.update_scroll();
    }

    pub fn move_to_top(&mut self) {
        self.selected_index = 0;
        self.update_scroll();
    }

    pub fn move_to_bottom(&mut self) {
        if !self.filtered_results.is_empty() {
            self.selected_index = self.filtered_results.len() - 1;
            self.update_scroll();
        }
    }

    fn update_scroll(&mut self) {
        let viewport_height = self.viewport_height;

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.selected_index.saturating_sub(viewport_height - 1);
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(tree_index) = self.get_selected_tree_index() {
            self.tree.toggle_state(tree_index);
        }
    }

    pub fn get_selected_tree_index(&self) -> Option<usize> {
        self.filtered_results
            .visible_items
            .get(self.selected_index)
            .copied()
    }

    pub fn select_all(&mut self) {
        for &tree_index in &self.filtered_results.visible_items {
            self.tree.set_state(tree_index, SelectionState::Included);
        }
    }

    pub fn select_none(&mut self) {
        for &tree_index in &self.filtered_results.visible_items {
            self.tree.set_state(tree_index, SelectionState::Excluded);
        }
    }

    pub fn invert_selection(&mut self) {
        for &tree_index in &self.filtered_results.visible_items {
            self.tree.toggle_state(tree_index);
        }
    }

    pub fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filtered_results();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.update_filtered_results();
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.update_filtered_results();
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn handle_escape(&mut self) {
        if self.mode == AppMode::Help {
            self.mode = AppMode::Main;
        } else if !self.search_query.is_empty() {
            // Clear search text if there is any
            self.search_query.clear();
            self.update_filtered_results();
        } else {
            // Quit if search is empty
            self.quit();
        }
    }

    pub fn get_stats(&self) -> AppStats {
        let total_files = self.tree.nodes.iter()
            .filter(|node| !node.is_directory && node.is_text_file)
            .count();

        let included_files = self.tree.get_all_included_files().len();

        let total_size: u64 = self.tree.get_all_included_files()
            .iter()
            .filter_map(|node| node.size)
            .sum();

        AppStats {
            total_files,
            included_files,
            total_size,
            filtered_count: self.filtered_results.len(),
        }
    }
}

#[derive(Debug)]
pub struct AppStats {
    pub total_files: usize,
    pub included_files: usize,
    pub total_size: u64,
    pub filtered_count: usize,
}

impl AppStats {
    pub fn format_size(&self) -> String {
        format_file_size(self.total_size)
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