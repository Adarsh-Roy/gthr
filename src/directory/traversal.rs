use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use anyhow::Result;
use globset::GlobSet;
use ignore::WalkBuilder;
use ignore::overrides::Override as IgnoreOverride;
use super::tree::{DirectoryTree, is_text_file};
use super::state::SelectionState;

#[derive(Debug)]
pub struct ScanEntry {
    pub path: PathBuf,
    pub is_directory: bool,
    pub parent_path: PathBuf,
    pub is_text_file: bool,
    pub file_size: Option<u64>,
}

#[derive(Debug)]
pub enum ScanMessage {
    Entry(ScanEntry),
    Complete,
    Error(String),
}

pub struct DirectoryTraverser {
    respect_gitignore: bool,
    show_hidden: bool,
    max_file_size: u64,
    include_all: bool,
    extra_text_ext: HashSet<String>,
    exclude_text_ext: HashSet<String>,
    hide_globset: Option<GlobSet>,
}

impl DirectoryTraverser {
    pub fn new(
        respect_gitignore: bool,
        show_hidden: bool,
        max_file_size: u64,
        include_all: bool,
        extra_text_ext: HashSet<String>,
        exclude_text_ext: HashSet<String>,
        hide_globset: Option<GlobSet>,
    ) -> Self {
        Self {
            respect_gitignore,
            show_hidden,
            max_file_size,
            include_all,
            extra_text_ext,
            exclude_text_ext,
            hide_globset,
        }
    }

    pub fn traverse(&self, root_path: &Path, overrides: Option<IgnoreOverride>) -> Result<DirectoryTree> {
        let mut tree = DirectoryTree::new(root_path.to_path_buf());

        let initial_state = if overrides.is_some() || self.include_all {
            SelectionState::Included
        } else {
            SelectionState::Excluded
        };
        tree.set_state(tree.root_index, initial_state);

        let mut builder = WalkBuilder::new(root_path);

        if !self.respect_gitignore {
            builder.git_ignore(false)
                   .git_global(false)
                   .git_exclude(false);
        }

        builder.hidden(!self.show_hidden);

        if let Some(ov) = overrides {
            builder.overrides(ov);
        }

        if let Some(ref hide_set) = self.hide_globset {
            let root = root_path.to_path_buf();
            let hide = hide_set.clone();
            builder.filter_entry(move |entry| {
                let path = entry.path();
                if path == root.as_path() {
                    return true;
                }
                let relative = path.strip_prefix(&root).unwrap_or(path);
                let name = entry.file_name().to_string_lossy();
                !hide.is_match(relative) && !hide.is_match(name.as_ref())
            });
        }

        let walker = builder.build();

        for result in walker {
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let path = entry.path();

            if path == root_path {
                continue;
            }

            if !should_include_entry_by_path(path, self.show_hidden) {
                continue;
            }

            let is_directory = entry.file_type().map_or(false, |ft| ft.is_dir());
            let parent_path = path.parent().unwrap_or(root_path);

            if !is_directory {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.len() > self.max_file_size {
                        continue;
                    }

                    if let Some(node_index) = tree.add_node(path.to_path_buf(), is_directory, parent_path, &self.extra_text_ext, &self.exclude_text_ext) {
                        if let Some(node) = tree.get_node_mut(node_index) {
                            node.size = Some(metadata.len());
                        }
                        tree.set_state(node_index, initial_state);
                    }
                }
            } else if let Some(node_index) = tree.add_node(path.to_path_buf(), is_directory, parent_path, &self.extra_text_ext, &self.exclude_text_ext) {
                tree.set_state(node_index, initial_state);
            }
        }

        Ok(tree)
    }

    pub fn traverse_streaming(&self, root_path: &Path) -> (Receiver<ScanMessage>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel::<ScanMessage>();
        let root_path = root_path.to_path_buf();
        let respect_gitignore = self.respect_gitignore;
        let show_hidden = self.show_hidden;
        let max_file_size = self.max_file_size;
        let extra_text_ext = self.extra_text_ext.clone();
        let exclude_text_ext = self.exclude_text_ext.clone();
        let hide_globset = self.hide_globset.clone();

        let handle = thread::spawn(move || {
            let mut builder = WalkBuilder::new(&root_path);

            if !respect_gitignore {
                builder.git_ignore(false)
                       .git_global(false)
                       .git_exclude(false);
            }
            builder.hidden(!show_hidden);

            if let Some(ref hide_set) = hide_globset {
                let root = root_path.clone();
                let hide = hide_set.clone();
                builder.filter_entry(move |entry| {
                    let path = entry.path();
                    if path == root.as_path() {
                        return true;
                    }
                    let relative = path.strip_prefix(&root).unwrap_or(path);
                    let name = entry.file_name().to_string_lossy();
                    !hide.is_match(relative) && !hide.is_match(name.as_ref())
                });
            }

            let walker = builder.build();

            for result in walker {
                let entry = match result {
                    Ok(entry) => entry,
                    Err(_) => continue,
                };

                let path = entry.path();

                if path == root_path {
                    continue;
                }

                if !should_include_entry_by_path(path, show_hidden) {
                    continue;
                }

                let is_directory = entry.file_type().map_or(false, |ft| ft.is_dir());

                let (file_size, is_text) = if is_directory {
                    (None, false)
                } else {
                    match std::fs::metadata(path) {
                        Ok(metadata) => {
                            let size = metadata.len();
                            if size > max_file_size {
                                continue;
                            }
                            (Some(size), is_text_file(path, &extra_text_ext, &exclude_text_ext))
                        }
                        Err(_) => continue,
                    }
                };

                let parent_path = path.parent().unwrap_or(&root_path).to_path_buf();

                let scan_entry = ScanEntry {
                    path: path.to_path_buf(),
                    is_directory,
                    parent_path,
                    is_text_file: is_text,
                    file_size,
                };

                if tx.send(ScanMessage::Entry(scan_entry)).is_err() {
                    return;
                }
            }

            let _ = tx.send(ScanMessage::Complete);
        });

        (rx, handle)
    }
}

fn should_include_entry_by_path(path: &Path, show_hidden: bool) -> bool {
    if !show_hidden {
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') && name_str != "." && name_str != ".." {
                if !matches!(
                    name_str.as_ref(),
                    ".gitignore" | ".gitattributes" | ".editorconfig" | ".env" | ".env.example"
                ) {
                    return false;
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_directory_traversal() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();

        // Create test directory structure
        fs::create_dir(root_path.join("src"))?;
        fs::write(root_path.join("src").join("main.rs"), "fn main() {}")?;
        fs::write(root_path.join("README.md"), "# Test Project")?;
        fs::create_dir(root_path.join("target"))?;
        fs::write(root_path.join("target").join("debug"), "binary")?;

        let traverser = DirectoryTraverser::new(true, false, 1024 * 1024, false, HashSet::new(), HashSet::new(), None);
        let tree = traverser.traverse(root_path, None)?;

        assert!(tree.nodes.len() >= 3); // root, src, main.rs, README.md

        Ok(())
    }

    fn build_test_globset(patterns: &[&str]) -> Option<GlobSet> {
        use globset::GlobSetBuilder;
        let mut builder = GlobSetBuilder::new();
        for p in patterns {
            builder.add(globset::Glob::new(p).unwrap());
        }
        Some(builder.build().unwrap())
    }

    #[test]
    fn test_hide_globset_skips_matching_files() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();

        fs::create_dir(root_path.join("src"))?;
        fs::write(root_path.join("src").join("main.rs"), "fn main() {}")?;
        fs::write(root_path.join("README.md"), "# Test")?;
        fs::write(root_path.join("debug.log"), "log content")?;

        let hide = build_test_globset(&["*.log"]);
        let traverser = DirectoryTraverser::new(true, false, 1024 * 1024, false, HashSet::new(), HashSet::new(), hide);
        let tree = traverser.traverse(root_path, None)?;

        let paths: Vec<String> = tree.nodes.iter().map(|n| n.name.clone()).collect();
        assert!(!paths.contains(&"debug.log".to_string()));
        assert!(paths.contains(&"main.rs".to_string()));

        Ok(())
    }

    #[test]
    fn test_hide_globset_prevents_directory_descent() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let root_path = temp_dir.path();

        fs::create_dir_all(root_path.join("target").join("debug"))?;
        fs::write(root_path.join("target").join("debug").join("binary"), "bin")?;
        fs::create_dir(root_path.join("src"))?;
        fs::write(root_path.join("src").join("main.rs"), "fn main() {}")?;

        let hide = build_test_globset(&["target"]);
        let traverser = DirectoryTraverser::new(true, false, 1024 * 1024, false, HashSet::new(), HashSet::new(), hide);
        let tree = traverser.traverse(root_path, None)?;

        let paths: Vec<String> = tree.nodes.iter().map(|n| n.name.clone()).collect();
        assert!(!paths.contains(&"target".to_string()));
        assert!(!paths.contains(&"debug".to_string()));
        assert!(!paths.contains(&"binary".to_string()));
        assert!(paths.contains(&"main.rs".to_string()));

        Ok(())
    }
}
