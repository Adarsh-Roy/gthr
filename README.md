# Text Ingest CLI

A powerful CLI tool for directory text ingestion with interactive fuzzy finder capabilities, similar to gitingest. Quickly generate text ingests of directories with the ability to interactively choose what to include or exclude using a nice terminal user interface.

## Features

- **Interactive Fuzzy Finder**: Browse and search through files with a responsive TUI
- **Hierarchical Selection**: Including/excluding directories affects all children
- **Color-coded Feedback**:
  - 🟢 Green: Included files/directories
  - 🔴 Red: Excluded files/directories
  - 🟡 Yellow: Partially included (mixed children states)
- **Multiple Modes**: Pre-include everything or start with everything excluded
- **Smart File Detection**: Automatically identifies text files vs binary files
- **Configurable**: Respect .gitignore files, custom file size limits
- **Multiple Output Formats**: Markdown output with syntax highlighting

## Installation

### Build from Source

```bash
git clone <repository-url>
cd text-ingest-cli
cargo build --release
```

The binary will be available at `target/release/textingest`.

## Usage

### Interactive Mode (Default)

```bash
# Start interactive mode in current directory
textingest

# Start with all files pre-included
textingest --include-all

# Start with all files excluded (pick what to include)
textingest --exclude-all

# Specify a different root directory
textingest -r /path/to/directory

# Save output to a file
textingest -o output.md
```

### Non-Interactive Mode

```bash
# Generate output immediately with current settings
textingest --non-interactive --include-all -o output.md

# Process specific directory
textingest --non-interactive --include-all -r /path/to/project -o project_ingest.md
```

### Direct Mode

```bash
# Use include/exclude patterns (TODO: Not yet implemented)
textingest direct --include "*.rs" --exclude "target/*"
```

## Interactive Controls

Simple and intuitive - no modes to worry about!

### Search
- `Type any character` - Directly adds to search (letters, numbers, symbols - all work!)
- `Backspace` - Delete search character
- `Esc` - Clear search text (or quit if search is empty)

### Navigation
- `↑/↓` - Move up/down through files
- `←/→` - Alternative up/down navigation
- `Page Up/Page Down` - Page navigation
- `Home/End` - Go to top/bottom

### Selection
- `Enter` - Toggle selection of current item (✓/✗)

### Actions
- `Ctrl+Enter` - Generate output and quit
- `Ctrl+H` - Show help
- `Esc` - Clear search (or quit if search is empty)

### Key Features
- **No modes**: Everything is immediate and direct
- **Type anywhere**: All characters go to search automatically
- **Arrow navigation**: Only arrow keys navigate, no vim mappings
- **Visual feedback**: Clear cursor (►) and status indicators (✓/✗)

## Command Line Options

```
Options:
  -r, --root <ROOT>                    Root directory to process [default: .]
  -i, --include-all                    Pre-include all files and directories
  -e, --exclude-all                    Pre-exclude all files and directories
  -o, --output <OUTPUT>                Output file path
      --non-interactive                Skip interactive mode and use current selection
      --respect-gitignore              Respect .gitignore files [default: true]
      --max-file-size <MAX_FILE_SIZE>  Maximum file size to include (in bytes) [default: 1048576]
```

## Examples

### Basic Usage

```bash
# Interactive mode with current directory
textingest

# Generate ingest of a specific project
textingest -r ~/projects/my-app -o my-app-ingest.md --include-all
```

### Advanced Usage

```bash
# Exclude large files (limit to 512KB)
textingest --max-file-size 524288

# Ignore .gitignore rules
textingest --respect-gitignore false

# Quick non-interactive ingest
textingest --non-interactive --include-all
```

## Output Format

The tool generates a comprehensive Markdown report containing:

- **Header section** with metadata (root directory, file count, total size, generation time)
- **File list** with sizes
- **File contents** with appropriate syntax highlighting based on file extensions

Example output structure:
```markdown
# Text Ingest Report
**Root Directory:** /path/to/project
**Files Included:** 15
**Total Size:** 245.3 KB
**Generated:** 2024-01-15 10:30:45 UTC

## Included Files
- src/main.rs (2.1 KB)
- src/lib.rs (5.4 KB)
...

## src/main.rs
**Size:** 2.1 KB
**Path:** /path/to/project/src/main.rs

```rust
fn main() {
    println!("Hello, world!");
}
```
```

## Supported File Types

The tool automatically detects text files based on extensions:

**Programming Languages:** rs, py, js, ts, jsx, tsx, java, go, rb, php, swift, kt, etc.
**Web Technologies:** html, css, scss, sass, json, xml, yaml, yml
**Configuration:** toml, ini, conf, env, gitignore, dockerignore
**Documentation:** md, txt, rst
**Scripts:** sh, bash, zsh, fish, ps1, bat, cmd

## Project Structure

```
text-ingest-cli/
├── src/
│   ├── main.rs              # Main application entry point
│   ├── cli.rs               # Command line interface
│   ├── directory/           # Directory traversal and tree management
│   │   ├── mod.rs
│   │   ├── tree.rs          # Directory tree structure
│   │   ├── traversal.rs     # File system traversal
│   │   └── state.rs         # Include/exclude state management
│   ├── ui/                  # Terminal user interface
│   │   ├── mod.rs
│   │   ├── app.rs           # Application state
│   │   ├── interface.rs     # TUI rendering
│   │   ├── events.rs        # Event handling
│   │   └── colors.rs        # Color scheme
│   ├── fuzzy/               # Fuzzy matching functionality
│   │   ├── mod.rs
│   │   ├── matcher.rs       # Fuzzy matching algorithm
│   │   └── filter.rs        # Search filtering
│   ├── output/              # Output generation
│   │   ├── mod.rs
│   │   ├── formatter.rs     # Markdown formatting
│   │   └── writer.rs        # File writing
│   └── config/              # Configuration management
│       ├── mod.rs
│       └── settings.rs      # Settings and configuration
├── Cargo.toml
└── README.md
```

## Dependencies

- **clap**: Command line argument parsing
- **ratatui**: Rich terminal user interface framework
- **crossterm**: Cross-platform terminal manipulation
- **fuzzy-matcher**: Fuzzy string matching
- **walkdir**: Directory traversal
- **serde/toml**: Configuration serialization
- **chrono**: Date and time handling

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Similar Tools

- [gitingest](https://gitingest.com/) - Web-based git repository ingestion
- [tree](https://github.com/tree/tree) - Directory listing utility
- [fd](https://github.com/sharkdp/fd) - Fast file finder
- [fzf](https://github.com/junegunn/fzf) - Fuzzy finder

## Roadmap

- [ ] Configuration file support (.textingestrc)
- [ ] Custom include/exclude patterns (glob support)
- [ ] Multiple output formats (JSON, plain text)
- [ ] Streaming output for very large directories
- [ ] Preset configurations for different project types
- [ ] Plugin system for custom file processors
- [ ] Integration with version control systems
- [ ] Performance optimizations for huge directories
```

