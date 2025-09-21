use std::path::{Path, PathBuf};
use std::collections::HashMap;
use super::state::SelectionState;

#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub children: Vec<usize>, // Indices into the tree's nodes vector
    pub parent: Option<usize>,
    pub state: SelectionState,
    pub is_text_file: bool,
}

impl FileNode {
    pub fn new(path: PathBuf, is_directory: bool, parent: Option<usize>) -> Self {
        let name = path
            .file_name()
            .unwrap_or_else(|| path.as_os_str())
            .to_string_lossy()
            .to_string();

        Self {
            path,
            name,
            is_directory,
            size: None,
            children: Vec::new(),
            parent,
            state: SelectionState::default(),
            is_text_file: false,
        }
    }

    pub fn add_child(&mut self, child_index: usize) {
        self.children.push(child_index);
    }
}

#[derive(Debug)]
pub struct DirectoryTree {
    pub nodes: Vec<FileNode>,
    pub root_index: usize,
    pub path_to_index: HashMap<PathBuf, usize>,
}

impl DirectoryTree {
    pub fn new(root_path: PathBuf) -> Self {
        let mut nodes = Vec::new();
        let mut path_to_index = HashMap::new();

        let root_node = FileNode::new(root_path.clone(), true, None);
        nodes.push(root_node);
        path_to_index.insert(root_path, 0);

        Self {
            nodes,
            root_index: 0,
            path_to_index,
        }
    }

    pub fn add_node(&mut self, path: PathBuf, is_directory: bool, parent_path: &Path) -> Option<usize> {
        if self.path_to_index.contains_key(&path) {
            return self.path_to_index.get(&path).copied();
        }

        let parent_index = self.path_to_index.get(parent_path).copied()?;
        let node_index = self.nodes.len();

        let mut node = FileNode::new(path.clone(), is_directory, Some(parent_index));

        // Determine if it's a text file
        if !is_directory {
            node.is_text_file = is_text_file(&path);
        }

        self.nodes.push(node);
        self.path_to_index.insert(path, node_index);

        // Add this node as a child to its parent
        self.nodes[parent_index].add_child(node_index);

        Some(node_index)
    }

    pub fn get_node(&self, index: usize) -> Option<&FileNode> {
        self.nodes.get(index)
    }

    pub fn get_node_mut(&mut self, index: usize) -> Option<&mut FileNode> {
        self.nodes.get_mut(index)
    }

    pub fn set_state(&mut self, index: usize, state: SelectionState) {
        let parent_index = {
            if let Some(node) = self.nodes.get_mut(index) {
                node.state = state;
                node.parent
            } else {
                return;
            }
        };

        // Propagate state to children
        self.propagate_to_children(index, state);

        // Update parent state based on children
        if let Some(parent_index) = parent_index {
            self.update_parent_state(parent_index);
        }
    }

    fn propagate_to_children(&mut self, parent_index: usize, state: SelectionState) {
        if state == SelectionState::Partial {
            return; // Don't propagate partial state
        }

        let children: Vec<usize> = self.nodes[parent_index].children.clone();
        for child_index in children {
            if let Some(child) = self.nodes.get_mut(child_index) {
                child.state = state;
            }
            self.propagate_to_children(child_index, state);
        }
    }

    fn update_parent_state(&mut self, parent_index: usize) {
        let children: Vec<usize> = self.nodes[parent_index].children.clone();

        if children.is_empty() {
            return;
        }

        let mut included_count = 0;
        let mut excluded_count = 0;
        let mut partial_count = 0;

        for child_index in &children {
            if let Some(child) = self.nodes.get(*child_index) {
                match child.state {
                    SelectionState::Included => included_count += 1,
                    SelectionState::Excluded => excluded_count += 1,
                    SelectionState::Partial => partial_count += 1,
                }
            }
        }

        let new_state = if partial_count > 0 || (included_count > 0 && excluded_count > 0) {
            SelectionState::Partial
        } else if included_count > 0 {
            SelectionState::Included
        } else {
            SelectionState::Excluded
        };

        if let Some(parent) = self.nodes.get_mut(parent_index) {
            parent.state = new_state;
        }

        // Recursively update grandparent
        if let Some(grandparent_index) = self.nodes[parent_index].parent {
            self.update_parent_state(grandparent_index);
        }
    }

    pub fn toggle_state(&mut self, index: usize) {
        if let Some(node) = self.nodes.get(index) {
            let new_state = node.state.toggle();
            self.set_state(index, new_state);
        }
    }

    pub fn get_all_included_files(&self) -> Vec<&FileNode> {
        let mut included_files = Vec::new();
        self.collect_included_files(self.root_index, &mut included_files);
        included_files
    }

    fn collect_included_files<'a>(&'a self, index: usize, included_files: &mut Vec<&'a FileNode>) {
        if let Some(node) = self.nodes.get(index) {
            if node.state.is_included() && !node.is_directory && node.is_text_file {
                included_files.push(node);
            }

            // Recursively check children
            for child_index in &node.children {
                self.collect_included_files(*child_index, included_files);
            }
        }
    }
}

fn is_text_file(path: &Path) -> bool {
    if let Some(extension) = path.extension() {
        let ext = extension.to_string_lossy().to_lowercase();
        matches!(
            ext.as_str(),
            "txt" | "md" | "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "html" | "css" | "scss"
                | "sass" | "json" | "yaml" | "yml" | "toml" | "xml" | "csv" | "sql" | "sh"
                | "bash" | "zsh" | "fish" | "ps1" | "bat" | "cmd" | "dockerfile" | "makefile"
                | "cmake" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" | "hxx" | "java" | "kt"
                | "kts" | "scala" | "go" | "rb" | "php" | "swift" | "dart" | "lua" | "perl"
                | "r" | "jl" | "hs" | "elm" | "clj" | "cljs" | "ex" | "exs" | "erl" | "hrl"
                | "vim" | "vimrc" | "emacs" | "el" | "lisp" | "scm" | "rkt" | "ml" | "mli"
                | "fs" | "fsi" | "fsx" | "fsscript" | "pas" | "pp" | "inc" | "asm" | "s"
                | "config" | "conf" | "ini" | "properties" | "env" | "gitignore" | "gitattributes"
                | "dockerignore" | "editorconfig" | "eslintrc" | "prettierrc" | "babelrc"
                | "webpack" | "rollup" | "vite" | "package" | "lock" | "sum" | "mod"
        )
    } else {
        // Check for files without extensions that are commonly text
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy().to_lowercase();
            matches!(
                name.as_str(),
                "readme" | "license" | "changelog" | "authors" | "contributors" | "makefile"
                    | "dockerfile" | "vagrantfile" | "gemfile" | "rakefile" | "procfile"
                    | "cmakelists" | "build" | "configure" | "install" | "news" | "todo"
                    | "copying" | "manifest"
            )
        } else {
            false
        }
    }
}