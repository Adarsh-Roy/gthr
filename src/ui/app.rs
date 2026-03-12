use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use std::time::Instant;
use crate::output::formatter::format_file_size;
use crate::directory::state::SelectionState;
use crate::directory::traversal::ScanMessage;
use crate::directory::tree::DirectoryTree;
use crate::fuzzy::filter::{FilteredResults, filter_tree_nodes};
use crate::ui::colors::ColorScheme;

#[derive(Debug, Clone, PartialEq)]
pub enum ScanState {
    Scanning,
    Complete,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Main,
    Help,
    FileSave,
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
    pub file_save_input: String,
    pub pending_content: Option<String>,
    pub scan_state: ScanState,
    pub scan_receiver: Option<Receiver<ScanMessage>>,
    pub scan_handle: Option<JoinHandle<()>>,
    pub initial_state: SelectionState,
    pub notification: Option<(String, Instant)>,
    pub exit_message: Option<String>,
    last_filter_update: Option<Instant>,
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
            viewport_height: 20,
            file_save_input: String::new(),
            pending_content: None,
            scan_state: ScanState::Complete,
            scan_receiver: None,
            scan_handle: None,
            initial_state: SelectionState::Excluded,
            notification: None,
            exit_message: None,
            last_filter_update: None,
        };

        app.update_filtered_results();
        app
    }

    pub fn new_streaming(
        root_path: std::path::PathBuf,
        receiver: Receiver<ScanMessage>,
        handle: JoinHandle<()>,
        initial_state: SelectionState,
    ) -> Self {
        let mut tree = DirectoryTree::new(root_path);
        tree.set_state(tree.root_index, initial_state);

        Self {
            filtered_results: FilteredResults::new(),
            tree,
            selected_index: 0,
            scroll_offset: 0,
            search_query: String::new(),
            mode: AppMode::Main,
            color_scheme: ColorScheme::default(),
            should_quit: false,
            viewport_height: 20,
            file_save_input: String::new(),
            pending_content: None,
            scan_state: ScanState::Scanning,
            scan_receiver: Some(receiver),
            scan_handle: Some(handle),
            initial_state,
            notification: None,
            exit_message: None,
            last_filter_update: None,
        }
    }

    pub fn drain_scan_entries(&mut self) -> bool {
        let receiver = match &self.scan_receiver {
            Some(rx) => rx,
            None => return false,
        };

        let mut added_any = false;
        let mut scan_finished = false;

        loop {
            match receiver.try_recv() {
                Ok(ScanMessage::Entry(entry)) => {
                    self.tree.add_node_from_scan_entry(
                        entry.path,
                        entry.is_directory,
                        &entry.parent_path,
                        entry.is_text_file,
                        entry.file_size,
                        self.initial_state,
                    );
                    added_any = true;
                }
                Ok(ScanMessage::Complete) => {
                    self.scan_state = ScanState::Complete;
                    scan_finished = true;
                    break;
                }
                Ok(ScanMessage::Error(msg)) => {
                    self.scan_state = ScanState::Error(msg);
                    scan_finished = true;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.scan_state = ScanState::Complete;
                    scan_finished = true;
                    break;
                }
            }
        }

        if added_any {
            // Always rebuild on scan completion; throttle to every 200ms while scanning
            let should_rebuild = if scan_finished {
                true
            } else {
                match self.last_filter_update {
                    None => true,
                    Some(last) => last.elapsed().as_millis() >= 200,
                }
            };

            if should_rebuild {
                self.update_filtered_results();
                self.last_filter_update = Some(Instant::now());
            }
        }

        added_any
    }

    pub fn is_scanning(&self) -> bool {
        self.scan_state == ScanState::Scanning
    }

    pub fn update_filtered_results(&mut self) {
        self.filtered_results = filter_tree_nodes(&self.tree, &self.search_query);

        // Reset scroll position when search changes
        self.scroll_offset = 0;

        // Adjust selected index if it's out of bounds
        if self.selected_index >= self.filtered_results.len() && !self.filtered_results.is_empty() {
            self.selected_index = self.filtered_results.len() - 1;
        } else if self.filtered_results.is_empty() {
            self.selected_index = 0;
        }

        self.update_scroll();
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.update_scroll_for_move_up();
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.filtered_results.len() {
            self.selected_index += 1;
            self.update_scroll_for_move_down();
        }
    }

    pub fn half_page_up(&mut self) {
        let half_page = self.viewport_height / 2;
        let old_index = self.selected_index;
        self.selected_index = self.selected_index.saturating_sub(half_page);

        if old_index != self.selected_index {
            self.scroll_offset = self.scroll_offset.saturating_sub(half_page);
        }
    }

    pub fn half_page_down(&mut self) {
        let half_page = self.viewport_height / 2;
        let old_index = self.selected_index;
        self.selected_index =
            (self.selected_index + half_page).min(self.filtered_results.len().saturating_sub(1));

        if old_index != self.selected_index {
            let max_scroll = self.filtered_results.len().saturating_sub(self.viewport_height);
            self.scroll_offset = (self.scroll_offset + half_page).min(max_scroll);
        }
    }

    pub fn page_up(&mut self) {
        let page_size = self.viewport_height.saturating_sub(1);
        let old_index = self.selected_index;
        self.selected_index = self.selected_index.saturating_sub(page_size);

        if old_index != self.selected_index {
            // Scroll to show the selected item at the top of the viewport
            self.scroll_offset = self.selected_index;
        }
    }

    pub fn page_down(&mut self) {
        let page_size = self.viewport_height.saturating_sub(1);
        let old_index = self.selected_index;
        self.selected_index =
            (self.selected_index + page_size).min(self.filtered_results.len().saturating_sub(1));

        if old_index != self.selected_index {
            // Scroll to show the selected item at the bottom of the viewport
            if self.selected_index >= self.viewport_height {
                self.scroll_offset = self.selected_index.saturating_sub(self.viewport_height - 1);
            } else {
                self.scroll_offset = 0;
            }
        }
    }

    pub fn move_to_top(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn move_to_bottom(&mut self) {
        if !self.filtered_results.is_empty() {
            self.selected_index = self.filtered_results.len() - 1;

            // Position the last item at the bottom of the viewport
            if self.selected_index >= self.viewport_height {
                self.scroll_offset = self.selected_index.saturating_sub(self.viewport_height - 1);
            } else {
                self.scroll_offset = 0;
            }
        }
    }

    fn update_scroll_for_move_up(&mut self) {
        // If the selected index is now above the visible area, scroll up
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        }
    }

    fn update_scroll_for_move_down(&mut self) {
        // If the selected index is now below the visible area, scroll down
        if self.selected_index >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = self.selected_index.saturating_sub(self.viewport_height - 1);
        }
    }

    fn update_scroll(&mut self) {
        // General scroll update - ensures selected item is visible
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = self.selected_index.saturating_sub(self.viewport_height - 1);
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

    pub fn add_search_char(&mut self, c: char) {
        self.search_query.push(c);
        self.update_filtered_results();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
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
        } else if self.mode == AppMode::FileSave {
            self.mode = AppMode::Main;
            self.file_save_input.clear();
            self.pending_content = None;
        } else if !self.search_query.is_empty() {
            // Clear search text if there is any
            self.search_query.clear();
            self.update_filtered_results();
        } else {
            // Quit if search is empty
            self.quit();
        }
    }

    pub fn start_file_save(&mut self, content: String) {
        self.pending_content = Some(content);
        self.file_save_input.clear();
        self.mode = AppMode::FileSave;
    }

    pub fn add_file_save_char(&mut self, c: char) {
        if self.mode == AppMode::FileSave {
            self.file_save_input.push(c);
        }
    }

    pub fn file_save_backspace(&mut self) {
        if self.mode == AppMode::FileSave {
            self.file_save_input.pop();
        }
    }

    pub fn set_notification(&mut self, msg: String) {
        self.notification = Some((msg, Instant::now()));
    }

    pub fn get_stats(&self) -> AppStats {
        let included_nodes = self.tree.get_all_included_files();
        let included_files = included_nodes.len();
        let total_size: u64 = included_nodes.iter().filter_map(|n| n.size).sum();

        let total_files = self
            .tree
            .nodes
            .iter()
            .filter(|node| !node.is_directory && node.is_text_file)
            .count();

        AppStats {
            total_files,
            included_files,
            total_size,
        }
    }
}

#[derive(Debug)]
pub struct AppStats {
    pub total_files: usize,
    pub included_files: usize,
    pub total_size: u64,
}

impl AppStats {
    pub fn format_size(&self) -> String {
        format_file_size(self.total_size)
    }
}

