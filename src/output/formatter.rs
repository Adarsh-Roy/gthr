use crate::directory::tree::{DirectoryTree, FileNode};
use anyhow::Result;
use std::fs;

pub struct OutputFormatter {
    include_metadata: bool,
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
            include_metadata: true,
            include_line_numbers: false,
        }
    }

    pub fn with_metadata(mut self, include_metadata: bool) -> Self {
        self.include_metadata = include_metadata;
        self
    }

    pub fn with_line_numbers(mut self, include_line_numbers: bool) -> Self {
        self.include_line_numbers = include_line_numbers;
        self
    }

    pub fn format_output(&self, tree: &DirectoryTree) -> Result<String> {
        let included_files = tree.get_all_included_files();
        let mut output = String::new();

        if self.include_metadata {
            // Add header
            output.push_str(&self.format_header(tree, &included_files)?);
            output.push_str("\n\n");
        }

        // Add file contents
        for (index, file_node) in included_files.iter().enumerate() {
            if index > 0 {
                output.push_str("\n\n");
            }
            output.push_str(&self.format_file(tree, file_node)?);
        }

        Ok(output)
    }

    fn format_header(&self, tree: &DirectoryTree, included_files: &[&FileNode]) -> Result<String> {
        let root_path = &tree.nodes[tree.root_index].path;
        let total_size: u64 = included_files.iter().filter_map(|node| node.size).sum();

        let mut header = String::new();
        header.push_str(&format!("# Text Ingest Report\n"));
        header.push_str(&format!("**Root Directory:** {}\n", root_path.display()));
        header.push_str(&format!("**Files Included:** {}\n", included_files.len()));
        header.push_str(&format!(
            "**Total Size:** {}\n",
            format_file_size(total_size)
        ));
        header.push_str(&format!(
            "**Generated:** {}\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        if self.include_metadata {
            header.push_str("\n## Included Files\n");
            for file_node in included_files {
                let relative_path = file_node
                    .path
                    .strip_prefix(root_path)
                    .unwrap_or(&file_node.path);
                let size_str = file_node
                    .size
                    .map(format_file_size)
                    .unwrap_or_else(|| "Unknown".to_string());
                header.push_str(&format!("- {} ({})\n", relative_path.display(), size_str));
            }
        }

        Ok(header)
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

        if self.include_metadata {
            if let Some(size) = file_node.size {
                output.push_str(&format!("**Size:** {}\n", format_file_size(size)));
            }
            output.push_str(&format!("**Path:** {}\n", file_node.path.display()));
            output.push_str("\n");
        }

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

