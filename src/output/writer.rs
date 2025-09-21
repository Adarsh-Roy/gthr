use std::fs;
use std::path::Path;
use std::io::{self, Write};
use anyhow::Result;
use arboard::Clipboard;
use crate::directory::tree::DirectoryTree;
use super::formatter::OutputFormatter;

pub struct OutputWriter {
    formatter: OutputFormatter,
}

impl Default for OutputWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputWriter {
    pub fn new() -> Self {
        Self {
            formatter: OutputFormatter::new(),
        }
    }

    pub fn with_formatter(mut self, formatter: OutputFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn write_to_file(&self, tree: &DirectoryTree, output_path: &Path) -> Result<()> {
        let content = self.formatter.format_output(tree)?;

        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(output_path, content)?;
        Ok(())
    }

    pub fn write_to_stdout(&self, tree: &DirectoryTree) -> Result<()> {
        let content = self.formatter.format_output(tree)?;
        print!("{}", content);
        Ok(())
    }

    pub fn write_to_clipboard_or_prompt(&self, tree: &DirectoryTree) -> Result<()> {
        let content = self.formatter.format_output(tree)?;
        const MAX_CLIPBOARD_SIZE: usize = 1024 * 1024; // 1MB

        if content.len() <= MAX_CLIPBOARD_SIZE {
            match self.try_write_to_clipboard(&content) {
                Ok(()) => {
                    println!("✓ Output copied to clipboard ({} bytes)", content.len());
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("⚠ Failed to copy to clipboard: {}", e);
                    eprintln!("Falling back to file prompt...");
                }
            }
        }

        // Either too large or clipboard failed - prompt for filename
        self.prompt_and_save_to_file(tree, &content)
    }

    fn try_write_to_clipboard(&self, content: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        clipboard.set_text(content)?;
        Ok(())
    }

    fn prompt_and_save_to_file(&self, tree: &DirectoryTree, content: &str) -> Result<()> {
        if content.len() > 1024 * 1024 {
            println!("⚠ Output is too large for clipboard ({} bytes > 1MB)", content.len());
        }

        print!("📁 Enter filename to save output (or press Enter for default): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let filename = if input.is_empty() {
            Self::generate_default_filename(tree)
        } else {
            // Add .md extension if not present
            if input.ends_with(".md") {
                input.to_string()
            } else {
                format!("{}.md", input)
            }
        };

        let path = Path::new(&filename);
        self.write_to_file(tree, path)?;
        println!("✓ Output saved to: {}", path.display());
        Ok(())
    }

    pub fn generate_default_filename(tree: &DirectoryTree) -> String {
        let root_name = tree.nodes[tree.root_index]
            .path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("directory"))
            .to_string_lossy();

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        format!("{}_ingest_{}.md", root_name, timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;

    #[test]
    fn test_generate_default_filename() {
        let temp_dir = TempDir::new().unwrap();
        let tree = DirectoryTree::new(temp_dir.path().to_path_buf());

        let filename = OutputWriter::generate_default_filename(&tree);
        assert!(filename.ends_with(".md"));
        assert!(filename.contains("ingest"));
    }
}