# Text Ingest CLI

A powerful CLI tool for directory text ingestion, similar to gitingest web app, with interactive fuzzy finder capabilities with the ability to interactively choose what to include or exclude using a modern terminal user interface.

## Features

- **Interactive Fuzzy Finder**: Browse and search through files with a responsive TUI
- **Hierarchical Selection**: Including/excluding directories affects all children
- **Color-coded Feedback**:
  - 🟢 Green: Included files/directories
  - 🔴 Red: Excluded files/directories
  - 🟡 Yellow: Partially included (mixed children states)
- **Multiple Modes**: Pre-include everything or start with everything excluded
- **Smart File Detection**: Automatically identifies text files vs binary files
- **Configurable**: Set max file size limit (default 1 MB), ignore or respect .gitignore
- **Smart Output**: Markdown output with syntax highlighting, automatically copies to clipboard
- **Vim-like Controls**: Vim-like navigation support (Ctrl+J/K) alongside arrow keys

## Installation

### Homebrew (Recommended)

```bash
# Add the tap and install
brew tap adarsh-roy/gthr
brew install gthr
```

Or install directly:
```bash
brew install adarsh-roy/gthr/gthr
```

### Build from Source

```bash
git clone https://github.com/Adarsh-Roy/gthr.git
cd gthr
cargo build --release
```

The binary will be available at `target/release/gthr`.

## Usage

### Interactive Mode (Default)

```bash
# Start interactive mode in current directory
gthr

# Start with all files pre-included
gthr --include-all

# Start with all files excluded (pick what to include, ths is the default)
gthr --exclude-all

# Specify a different root directory
gthr -r /path/to/directory

# Save output to a file (it copies to clipboard regardless)
gthr -o output.md
```

### Non-Interactive Mode

```bash
# Generate output immediately with current settings
gthr --non-interactive --include-all -o output.md

# Process specific directory
gthr --non-interactive --include-all -r /path/to/project -o project_ingest.md
```

### Direct Mode (WIP)

```bash
# Use include/exclude patterns (TODO: Not yet implemented)
gthr direct --include "*.rs" --exclude "target/*"
```

## Interactive Controls

Simple and intuitive - no modes to worry about!

### Search
- `Type any character` - Directly adds to search (letters, numbers, symbols)
- `Backspace` - Delete search character
- `Esc` - Clear search text (or quit if search is empty)

### Navigation
- `↑/↓` - Move up/down through files
- `←/→` - Alternative up/down navigation
- `Ctrl+J/Ctrl+K` - Vim-like up/down navigation
- `Page Up/Page Down` - Page navigation
- `Home/End` - Go to top/bottom

### Selection
- `Enter` - Toggle selection of current item (✓/✗)

### Actions
- `Ctrl+E` - Export output and quit
- `Ctrl+H` - Show help
- `Esc` - Clear search (or quit if search is empty)

### Output Behavior
- **Default**: Copies output to clipboard (up to 1MB)
- **Large files**: Prompts for filename if output exceeds 1MB
- **Manual save**: Use `-o filename.md` to save directly to file

## Command Line Options

```
Options:
  -r, --root <ROOT>                    Root directory to process [default: .]
  -i, --include-all                    Pre-include all files and directories
  -e, --exclude-all                    Pre-exclude all files and directories
  -o, --output <OUTPUT>                Output file path
      --non-interactive                Skip interactive mode and use current selection
      --max-file-size <MAX_FILE_SIZE>  Maximum file size to include (in bytes) [default: 1048576]
```

## Advanced Usage

```bash
# Exclude large files (limit to 512KB)
gthr --max-file-size 524288

# Quick non-interactive export
gthr --non-interactive --include-all
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
- [ ] Custom include/exclude patterns (glob support, regex support in the search area)
- [ ] Multiple output formats (JSON, plain text)
- [ ] Preset configurations for different project types
- [ ] Performance optimizations for huge directories
```

