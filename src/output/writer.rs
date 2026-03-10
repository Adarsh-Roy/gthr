use super::formatter::OutputFormatter;
use crate::directory::tree::DirectoryTree;
use anyhow::Result;
use std::fs;
use std::path::Path;

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

    pub fn generate_default_filename(tree: &DirectoryTree) -> String {
        let root_name = tree.nodes[tree.root_index]
            .path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("directory"))
            .to_string_lossy();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let secs_per_day = 86400;
        let secs_per_hour = 3600;
        let secs_per_min = 60;
        let total_days = now / secs_per_day;
        let time_of_day = now % secs_per_day;
        let hour = time_of_day / secs_per_hour;
        let minute = (time_of_day % secs_per_hour) / secs_per_min;
        let second = time_of_day % secs_per_min;

        // Compute year/month/day from days since epoch
        let (year, month, day) = {
            // Shift to March-based year to simplify leap year handling
            let days = total_days + 719468; // days from 0000-03-01 to 1970-01-01
            let era = days / 146097;
            let doe = days - era * 146097;
            let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
            let y = yoe + era * 400;
            let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
            let mp = (5 * doy + 2) / 153;
            let d = doy - (153 * mp + 2) / 5 + 1;
            let m = if mp < 10 { mp + 3 } else { mp - 9 };
            let y = if m <= 2 { y + 1 } else { y };
            (y, m, d)
        };

        format!(
            "{}_ingest_{}{:02}{:02}_{:02}{:02}{:02}.md",
            root_name, year, month, day, hour, minute, second
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_generate_default_filename() {
        let temp_dir = TempDir::new().unwrap();
        let tree = DirectoryTree::new(temp_dir.path().to_path_buf());

        let filename = OutputWriter::generate_default_filename(&tree);
        assert!(filename.ends_with(".md"));
        assert!(filename.contains("ingest"));
    }
}

