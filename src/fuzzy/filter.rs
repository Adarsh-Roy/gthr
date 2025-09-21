use super::matcher::{search_items, MatchResult};
use crate::directory::tree::{DirectoryTree, FileNode};

pub struct FilteredResults {
    pub matches: Vec<MatchResult>,
    pub visible_items: Vec<usize>, // Indices into the original tree
}

impl FilteredResults {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            visible_items: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    pub fn len(&self) -> usize {
        self.matches.len()
    }
}

pub fn filter_tree_nodes(tree: &DirectoryTree, query: &str) -> FilteredResults {
    // Collect all nodes that should be searchable
    let searchable_nodes: Vec<(usize, &FileNode)> = tree
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| {
            // Include directories and text files
            node.is_directory || node.is_text_file
        })
        .collect();

    // Extract text for fuzzy matching (use relative path from root)
    let node_texts: Vec<String> = searchable_nodes
        .iter()
        .map(|(_, node)| {
            // Create a display path relative to the root
            if let Ok(relative_path) = node.path.strip_prefix(&tree.nodes[tree.root_index].path) {
                relative_path.to_string_lossy().to_string()
            } else {
                node.name.clone()
            }
        })
        .collect();

    // Perform fuzzy search
    let matches = search_items(&node_texts, query, |text| text.as_str());

    // Map results back to tree indices
    let visible_items: Vec<usize> = matches
        .iter()
        .map(|match_result| searchable_nodes[match_result.item_index].0)
        .collect();

    FilteredResults {
        matches,
        visible_items,
    }
}

pub fn get_node_display_path(tree: &DirectoryTree, node_index: usize) -> String {
    if let Some(node) = tree.get_node(node_index) {
        if let Ok(relative_path) = node.path.strip_prefix(&tree.nodes[tree.root_index].path) {
            relative_path.to_string_lossy().to_string()
        } else {
            node.name.clone()
        }
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_filter_empty_query() {
        let temp_dir = TempDir::new().unwrap();
        let tree = DirectoryTree::new(temp_dir.path().to_path_buf());

        let results = filter_tree_nodes(&tree, "");
        assert_eq!(results.len(), 1); // Should include the root directory
    }
}