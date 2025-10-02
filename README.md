# gthr

A CLI tool for directory text ingestion, similar to gitingest web app,
with interactive fuzzy finder capabilities with the ability to interactively choose
what to include or exclude using a modern terminal user interface.

## Features

- **Smart Output**: Markdown output with syntax highlighting. Automatically copies to clipboard when you export. Optionally, saves to a file.
- **Interactive Fuzzy Finder**: Browse and search through files with a responsive TUI
- **Hierarchical Selection**: Including/excluding directories affects all children
- **Color-coded Feedback**:
  - 🟢 Green: Included files/directories
  - 🔴 Red: Excluded files/directories
  - 🟡 Yellow: Partially included (mixed children states)
- **Two Modes**: Interactive mode with fuzzy finder or direct mode with pattern matching
- **Smart File Detection**: Automatically identifies text files vs binary files
- **Configurable**: Set max file size limit (default 1 MB), ignore or respect `.gitignore`
- **Pattern Matching**: Supports glob patterns for include/exclude (e.g., `*.rs`, `**/*`)
- **Vim-like Controls**: Vim-like navigation (`Ctrl+J`/`Ctrl-K`) alongside arrow keys

## Installation

### Homebrew

```bash
brew install adarsh-roy/gthr/gthr
```

### Build From Source

**NOTE**: You must have `cargo` installed.

```bash
git clone https://github.com/Adarsh-Roy/gthr.git
cd gthr
cargo build --release
```

The binary will be available at `target/release/gthr`. To install it in a directory you can run:

```bash
install -m 755 ./target/release/gthr /path/to/bin
```

Make sure that `/path/to/bin` is in your `$PATH` environment variable.

## Usage

### Interactive Mode

By default, `gthr` runs in _Interactive Mode_.

```bash
# Start interactive mode in current directory
gthr

# Explicitly run interactive mode
gthr interactive

# Start with all files pre-included
gthr -I

# Start with all files excluded (pick what to include, this is the default)
gthr -E

# Specify a different root directory
gthr -r /path/to/directory

# Save output to a file (it copies to clipboard regardless)
gthr -o output.md

# Ignore .gitignore files (include everything, even ignored files)
gthr -g false

# Start interactive mode with patterns pre-applied
gthr -i "*.rs" -e "target/*" interactive

# Complex example: include Rust files, exclude tests, ignore gitignore
gthr -g false -i "*.rs" -e "*test*" -o rust_code.md interactive
```

### Direct Mode

Generate output immediately without interactive interface using include/exclude patterns.

```bash
# Include only Rust files
gthr -i "*.rs" direct

# Include multiple patterns
gthr -i "*.rs" -i "*.toml" direct

# Include with exclusions
gthr -i "**/*" -e "target/*" -e "*.log" direct

# Save to file with gitignore disabled
gthr -g false -i "*.rs" -o output.md direct

# Process specific directory
gthr -r /path/to/project -i "src/**/*.rs" direct

# Pre-include everything then exclude specific patterns
gthr -I -e "target/*" -e "*.log" direct
```

## Interactive Controls

### Search
- `Type any character` - Directly adds to search (letters, numbers, symbols)
- `Backspace` - Delete search character
- `Esc` - Clear search text (or quit if search is empty)

### Navigation
- `↑/↓` - Move up/down through files
- `←/→` - Alternative up/down navigation
- `Ctrl+J/Ctrl+K` - Vim-like up/down navigation

### Selection
- `Enter` - Toggle selection of current item (✓/✗)

### Actions
- `Ctrl+E` - Export output and quit
- `Ctrl+H` - Show help
- `Esc` - Clear search (or quit if search is empty)

### Output Behavior
- **Default**: Copies output to clipboard (up to 2MB)
- **Large files**: Shows save dialog if output exceeds 2MB
- **Manual save**: Use `-o filename.md` to save directly to file

## Command Line Options

```
Commands:
  interactive  Run the interactive fuzzy finder interface (default)
  direct       Generate text ingest directly without interaction
  help         Print help information

Global Options:
  -r, --root <ROOT>                    Root directory to process [default: .]
  -I, --include-all                    Pre-include all files and directories
  -E, --exclude-all                    Pre-exclude all files and directories
  -i, --include <PATTERN>              Pattern to include files (glob pattern)
  -e, --exclude <PATTERN>              Pattern to exclude files (glob pattern)
  -o, --output <OUTPUT>                Output file path
  -g, --respect-gitignore <BOOL>       Respect .gitignore files [default: true]
      --max-file-size <SIZE>           Maximum file size to include (in bytes) [default: 1048576]
  -h, --help                           Print help
  -V, --version                        Print version
```

## Usage examples

```bash
# Exclude large files (limit to 512KB)
gthr --max-file-size 524288

# Quick direct export of all Rust files, ignoring .gitignore
gthr -g false -i "**/*.rs" -o rust_files.md direct

# Include everything but exclude build artifacts
gthr -i "**/*" -e "target/*" -e "node_modules/*" -e "*.log" direct

# Process specific directory with custom file size limit
gthr -r /path/to/project --max-file-size 2097152 -i "src/**/*" direct

# Interactive mode with patterns: start with Rust files included, target excluded
gthr -i "*.rs" -e "target/*" interactive

# Include source files but exclude tests and builds
gthr -I -e "*test*" -e "target/*" -e "build/*" -e "dist/*" direct

# Multiple include patterns with exclusions
gthr -i "*.rs" -i "*.toml" -i "*.md" -e "target/*" -o project_overview.md direct
```

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## Similar Tools and Differences

- [gitingest](https://gitingest.com/) - Web-based git repository ingestion (awesome website). It has a cli version as well.
    - It doesn't have interactive include/exclude with fuzzy matching.
    - The digest is either printed to stdout or saved in a file, there's no "copy to clipboard and paste right away" option.

## Roadmap

- [ ] Configuration file support (global and local)
- [ ] Regex support in search bar in interactive mode
- [ ] Performance optimizations for huge directories

