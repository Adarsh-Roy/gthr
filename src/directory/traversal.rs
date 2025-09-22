use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use anyhow::Result;
use super::tree::DirectoryTree;
use super::state::SelectionState;

pub struct DirectoryTraverser {
    respect_gitignore: bool,
    max_file_size: u64,
    include_all: bool,
}

impl DirectoryTraverser {
    pub fn new(respect_gitignore: bool, max_file_size: u64, include_all: bool) -> Self {
        Self {
            respect_gitignore,
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

        let walker = WalkDir::new(root_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| self.should_include_entry(entry));

        for entry in walker {
            if entry.path() == root_path {
                continue; // Skip root as it's already added
            }

            let is_directory = entry.file_type().is_dir();
            let parent_path = entry.path().parent().unwrap_or(root_path);

            if let Some(node_index) = tree.add_node(entry.path().to_path_buf(), is_directory, parent_path) {
                // Set file size for files
                if !is_directory {
                    if let Ok(metadata) = entry.metadata() {
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

    fn should_include_entry(&self, entry: &DirEntry) -> bool {
        let path = entry.path();

        // Skip hidden files and directories if not explicitly included
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

        // Check file size for regular files
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.len() > self.max_file_size {
                    return false;
                }
            }
        }

        // TODO: Implement gitignore checking
        if self.respect_gitignore {
            // For now, skip common build/cache directories
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy();
                if matches!(
                    name_str.as_ref(),
                    "node_modules" | "target" | "build" | "dist" | ".git" | ".svn" | ".hg"
                        | "__pycache__" | ".pytest_cache" | ".coverage" | "coverage"
                        | ".nyc_output" | "vendor" | "bin" | "obj" | ".vscode" | ".idea"
                        | ".vs" | "*.tmp" | "*.temp" | "*.log"
                ) {
                    return false;
                }
            }
        }

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

        let traverser = DirectoryTraverser::new(true, 1024 * 1024, false);
        let tree = traverser.traverse(root_path)?;

        assert!(tree.nodes.len() >= 3); // root, src, main.rs, README.md (target should be filtered)

        Ok(())
    }
}