use crate::directory::tree::{DirectoryTree, FileNode};
use anyhow::Result;
use std::fs;

pub struct OutputFormatter {
    include_line_numbers: bool,
}

impl Default for OutputFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter {
    pub fn new() -> Self {
        Self {
            include_line_numbers: false,
        }
    }

    pub fn with_line_numbers(mut self, include_line_numbers: bool) -> Self {
        self.include_line_numbers = include_line_numbers;
        self
    }

    pub fn format_output(&self, tree: &DirectoryTree) -> Result<String> {
        let included_files = tree.get_all_included_files();
        let mut output = String::new();

        // Add file contents
        for (index, file_node) in included_files.iter().enumerate() {
            if index > 0 {
                output.push_str("\n\n");
            }
            output.push_str(&self.format_file(tree, file_node)?);
        }

        Ok(output)
    }

    fn format_file(&self, tree: &DirectoryTree, file_node: &FileNode) -> Result<String> {
        let root_path = &tree.nodes[tree.root_index].path;
        let relative_path = file_node
            .path
            .strip_prefix(root_path)
            .unwrap_or(&file_node.path);

        let mut output = String::new();

        // Always include file header for context
        output.push_str(&format!("# {}\n\n", relative_path.display()));

        // File content
        match fs::read_to_string(&file_node.path) {
            Ok(content) => {
                output.push_str("```");

                // Add language hint based on file extension
                if let Some(ext) = file_node.path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    let language = match ext_str.as_str() {
                        "rs" => "rust",
                        "py" => "python",
                        "js" => "javascript",
                        "ts" => "typescript",
                        "jsx" => "jsx",
                        "tsx" => "tsx",
                        "html" => "html",
                        "css" => "css",
                        "scss" | "sass" => "scss",
                        "json" => "json",
                        "yaml" | "yml" => "yaml",
                        "toml" => "toml",
                        "xml" => "xml",
                        "sql" => "sql",
                        "sh" | "bash" => "bash",
                        "c" => "c",
                        "cpp" | "cc" | "cxx" => "cpp",
                        "h" | "hpp" | "hxx" => "cpp",
                        "java" => "java",
                        "go" => "go",
                        "rb" => "ruby",
                        "php" => "php",
                        "swift" => "swift",
                        "kt" | "kts" => "kotlin",
                        "scala" => "scala",
                        "md" => "markdown",
                        "typ" => "typst",
                        _ => "",
                    };
                    output.push_str(language);
                }

                output.push('\n');

                if self.include_line_numbers {
                    for (line_num, line) in content.lines().enumerate() {
                        output.push_str(&format!("{:4} | {}\n", line_num + 1, line));
                    }
                } else {
                    output.push_str(&content);
                }

                output.push_str("\n```");
            }
            Err(e) => {
                output.push_str(&format!("*Error reading file: {}*", e));
            }
        }

        Ok(output)
    }
}

/// Format a byte count as a human-readable size string (e.g. "1.5 MB", "512 B").
pub fn format_file_size(size: u64) -> String {
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

