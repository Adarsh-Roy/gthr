# gthr

Gathers text content from a directory tree into clipboard or file. Local alternative to [gitingest](https://gitingest.com/) with a fuzzy-finding TUI.

<p align="center">
  <img src="./docs/gthr.png" alt="gthr"/>
</p>

## Installation

### Homebrew

```bash
brew install adarsh-roy/gthr/gthr
```

### Build from source

Requires `cargo`:

```bash
cargo install --git https://github.com/Adarsh-Roy/gthr --locked
```

## Usage

```bash
# Interactive mode (default) — all files start excluded
gthr

# Pre-include everything, then deselect what you don't need
gthr -I

# Direct mode — include only Rust files, output immediately
gthr -i "*.rs" direct

# Exclude build artifacts
gthr -I -e "target/*" -e "node_modules/*" direct

# Include specific paths (files or directories, relative or absolute)
gthr -p src/lib.rs -p tests/ direct

# Pre-select specific files in interactive mode
gthr -p src/lib.rs -p src/cli.rs

# Save to file instead of clipboard
gthr -o output.md

# Everything, including hidden files, ignoring .gitignore
gthr -I -H true -g false

# Clone a remote repo and browse in TUI
gthr --url https://github.com/user/repo

# Clone, gather Rust files, output directly
gthr --url https://github.com/user/repo -i "*.rs" direct

# Clone and keep the repo (CWD by default, or specify a directory)
gthr --url https://github.com/user/repo --keep
gthr --url https://github.com/user/repo --keep ~/projects
```

### Modes

**Interactive** (default): Browse the file tree in a TUI, fuzzy-search, and toggle files before exporting.

**Direct** (`gthr direct`): Applies flags and outputs without opening the TUI.

### Interactive controls

| Action | Keys |
|---|---|
| Search | Type any character |
| Clear search / quit | `Esc` |
| Navigate | `↑`/`↓`, `Ctrl+J`/`Ctrl+K` |
| Half-page | `Ctrl+D`/`Ctrl+U` |
| Full-page | `Ctrl+F`/`Ctrl+B` |
| Jump to top/bottom | `Ctrl+T`/`Ctrl+G` |
| Toggle selection | `Enter` |
| Export and quit | `Ctrl+E` |
| Help | `Ctrl+H` |

### Remote repositories

`--url` shallow-clones any git URL (`--depth 1`) into a temp directory, cleaned up on exit. `--keep` persists the clone (CWD by default, or pass a path). Uses your local `git`, so SSH keys and credential helpers work for private repos.

### Output behavior

- Copies to clipboard by default (up to `max_clipboard_size`).
- If output exceeds the clipboard limit, prompts to save to a file.
- `-o <path>` writes directly to a file.

## CLI reference

```
Options:
  -r, --root <ROOT>                Root directory [default: .]
      --url <URL>                  Git URL to clone and gather from
      --keep [DIR]                 Keep cloned repo (default: CWD, requires --url)
  -I, --include-all                Pre-include all files
  -i, --include <PATTERN>          Include glob pattern (repeatable)
  -e, --exclude <PATTERN>          Exclude glob pattern (repeatable)
      --hide-pattern <PATTERN>     Hide files from tree entirely (repeatable)
  -p, --path <PATH>                Explicit file/directory paths (repeatable)
  -o, --output <PATH>              Write output to file
  -g, --respect-gitignore <BOOL>   Respect .gitignore [default: true]
  -H, --show-hidden <BOOL>         Show dotfiles [default: false]
      --max-file-size <BYTES>      Skip files larger than this [default: 2097152]
  -h, --help                       Print help
  -V, --version                    Print version

Commands:
  interactive  TUI fuzzy finder (default)
  direct       Output without interaction
```

## Configuration

gthr supports both a global config and per-project local configs. A global config is auto-created at `~/.config/gthr.toml` on first run. Respects `XDG_CONFIG_HOME`.

To configure a specific project, create a `gthr.toml` in the project root directory (the `--root` directory, defaults to CWD).

**Priority** (highest wins): CLI flags > local `gthr.toml` > global `~/.config/gthr.toml` > built-in defaults.

For any field, local wins over global. If only one defines it, that value is used.

### Config options

| Key | Default | Description |
|---|---|---|
| `max_file_size` | `2097152` (2 MB) | Skip files larger than this (bytes) |
| `max_clipboard_size` | `2097152` (2 MB) | Trigger file-save prompt above this size |
| `respect_gitignore` | `true` | Honor `.gitignore` rules during traversal |
| `show_hidden` | `false` | Include dotfiles and dot-directories |
| `extra_text_extensions` | `[]` | Extensions to always treat as text (e.g. `["mdx", "astro"]`) |
| `exclude_text_extensions` | `[]` | Extensions to never treat as text (e.g. `["min.js", "log"]`) |
| `include_patterns` | `[]` | Glob patterns to include files (e.g. `["src/**/*.rs", "*.toml"]`) |
| `exclude_patterns` | `[]` | Glob patterns to exclude files (e.g. `["target/**", "*.log"]`) |
| `hide_patterns` | `[]` | Glob patterns to remove from the tree (files won't appear in TUI or output) |

### Include / exclude patterns

Use `include_patterns` and `exclude_patterns` to pre-filter files in both interactive and direct modes. These use the same glob syntax as the `-i`/`-e` CLI flags.

```toml
# Local gthr.toml — only gather Rust and TOML files, skip build output
include_patterns = ["*.rs", "*.toml"]
exclude_patterns = ["target/**"]
```

When both include and exclude patterns are set, a file must match an include pattern **and** not match any exclude pattern to be selected. When only exclude patterns are set, matching files are excluded and everything else is left in its default state.

If CLI `-i`/`-e` flags are provided, they fully replace the corresponding config patterns.

### Hide patterns

`hide_patterns` removes files from the tree. They won't appear in the TUI or direct mode output. Excluded files are still visible but deselected; hidden files are gone.

```toml
hide_patterns = ["target/**", "node_modules"]
```

Stacks with `show_hidden`: hides additional files, does not reveal hidden ones. CLI `--hide-pattern` flags fully replace config `hide_patterns`.

### Text file detection overrides

gthr decides whether a file is "text" using ~60 hardcoded extensions plus content-based heuristics (UTF-8 check, null-byte detection). If a file isn't recognized or is wrongly classified, override it in config:

```toml
# Treat .prisma and .mdx as text
extra_text_extensions = ["prisma", "mdx"]

# Never treat .log or .min.js as text, even though they'd pass heuristics
exclude_text_extensions = ["log", "min.js"]
```

`exclude_text_extensions` takes priority. If an extension appears in both lists, it's excluded.

Extensions are case-insensitive and leading dots are stripped (`.RS`, `rs`, and `.rs` all match).

## Limitations

- **Large directories** — scanning happens in a background thread with streaming updates, but very large trees (100k+ files) will take time to fully load in interactive mode.

## Contributing

Issues, feature requests, and pull requests are welcome.

## License

MIT — see [LICENSE](./LICENSE).

## Similar tools

- [gitingest](https://gitingest.com/) — web-based repository ingestion. gthr runs locally with a TUI and clipboard output.
