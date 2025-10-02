use std::path::Path;
use anyhow::Result;
use ignore::WalkBuilder;
use super::tree::DirectoryTree;
use super::state::SelectionState;

pub struct DirectoryTraverser {
    respect_gitignore: bool,
    show_hidden: bool,
    max_file_size: u64,
    include_all: bool,
}

impl DirectoryTraverser {
    pub fn new(respect_gitignore: bool, show_hidden: bool, max_file_size: u64, include_all: bool) -> Self {
        Self {
            respect_gitignore,
            show_hidden,
            max_file_size,
            include_all,
        }
    }

    pub fn traverse(&self, root_path: &Path) -> Result<DirectoryTree> {
        let mut tree = DirectoryTree::new(root_path.to_path_buf());

        // Set initial state for root
        let initial_state = if self.include_all {
            SelectionState::Included
        } else {
            SelectionState::Excluded
        };
        tree.set_state(tree.root_index, initial_state);

        let mut builder = WalkBuilder::new(root_path);

        // Configure the walker based on our settings
        if !self.respect_gitignore {
            builder.git_ignore(false)
                   .git_global(false)
                   .git_exclude(false);
        }

        // Configure hidden files visibility
        builder.hidden(!self.show_hidden);

        // Build the walker and iterate
        let walker = builder.build();

        for result in walker {
            let entry = match result {
                Ok(entry) => entry,
                Err(_) => continue, // Skip entries we can't read
            };

            let path = entry.path();

            if path == root_path {
                continue; // Skip root as it's already added
            }

            // Apply our custom filtering
            if !self.should_include_entry_by_path(path) {
                continue;
            }

            let is_directory = entry.file_type().map_or(false, |ft| ft.is_dir());
            let parent_path = path.parent().unwrap_or(root_path);

            // Check file size before adding to tree
            if !is_directory {
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.len() > self.max_file_size {
                        // Skip files that are too large
                        continue;
                    }
                }
            }

            if let Some(node_index) = tree.add_node(path.to_path_buf(), is_directory, parent_path) {
                // Set file size for files
                if !is_directory {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        if let Some(node) = tree.get_node_mut(node_index) {
                            node.size = Some(metadata.len());
                        }
                    }
                }

                // Set initial state
                tree.set_state(node_index, initial_state);
            }
        }

        Ok(tree)
    }

    fn should_include_entry_by_path(&self, path: &Path) -> bool {
        // Skip hidden files and directories unless show_hidden is enabled
        if !self.show_hidden {
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') && name_str != "." && name_str != ".." {
                    // Allow some common config files
                    if !matches!(
                        name_str.as_ref(),
                        ".gitignore" | ".gitattributes" | ".editorconfig" | ".env" | ".env.example"
                    ) {
                        return false;
                    }
                }
            }
        }

        // Note: gitignore filtering is now handled by the ignore crate's WalkBuilder
        // File size filtering is handled in the main loop

        true
    }
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

        let traverser = DirectoryTraverser::new(true, false, 1024 * 1024, false);
        let tree = traverser.traverse(root_path)?;

        assert!(tree.nodes.len() >= 3); // root, src, main.rs, README.md

        Ok(())
    }
}