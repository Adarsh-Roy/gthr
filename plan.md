# CLI Directory Text Ingest Tool - Implementation Plan

## Project Overview
A CLI tool similar to gitingest that allows users to quickly generate text ingests of directories with an interactive fuzzy finder interface for include/exclude selection.

## Core Features
- Directory traversal and text extraction
- Interactive fuzzy finder (fzf-like) TUI
- Include/exclude selection with visual feedback
- Hierarchical selection (parent affects children)
- Pre-include/pre-exclude modes
- Color-coded feedback (green=included, red=excluded)

## Technology Stack
- **Language**: Rust (for performance and CLI tooling)
- **TUI Framework**: ratatui (for rich terminal UI)
- **Fuzzy Matching**: fuzzy-matcher crate
- **File System**: walkdir crate
- **CLI Framework**: clap for argument parsing

## Implementation Phases

### Phase 1: Project Setup and Basic CLI ✅ COMPLETED
- [x] Initialize Rust project with Cargo.toml
- [x] Add dependencies (clap, ratatui, crossterm, fuzzy-matcher, walkdir)
- [x] Create basic CLI structure with clap
- [x] Implement basic directory traversal
- [x] Add command-line flags:
  - `--include-all` / `-i`: Pre-include everything
  - `--exclude-all` / `-e`: Pre-exclude everything (pick what to include)
  - `--output` / `-o`: Output file path
  - `--root` / `-r`: Root directory to process

### Phase 2: Core Directory Processing ✅ COMPLETED
- [x] Implement directory tree structure
- [x] Create file/directory node representation
- [x] Add include/exclude state tracking
- [x] Implement hierarchical state propagation (parent → children)
- [x] Add file type filtering (text files only by default)
- [x] Implement gitignore-style pattern matching (basic version)

### Phase 3: Fuzzy Finder Implementation ✅ COMPLETED
- [x] Create fuzzy matching algorithm integration
- [x] Implement search functionality
- [x] Add keyboard navigation (arrow keys, vim-style)
- [x] Create filtered view based on search query
- [x] Add selection highlighting

### Phase 4: TUI Interface ✅ COMPLETED
- [x] Design main interface layout
- [x] Implement directory tree visualization
- [x] Add color coding system:
  - Green: Included items
  - Red: Excluded items
  - Gray: Inherited state
  - Yellow: Partially included (some children excluded)
- [x] Create status bar with instructions
- [x] Add search input field
- [x] Implement real-time filtering display

### Phase 5: Interaction Logic ✅ COMPLETED
- [x] Implement keyboard event handling
- [x] Add toggle functionality (Space/Enter to toggle inclusion)
- [x] Implement hierarchical selection logic:
  - Including parent includes all children
  - Excluding parent excludes all children
  - Mixed states show partial inclusion
- [x] Add batch operations (select all visible, invert selection)
- [ ] Implement undo/redo functionality (NOT IMPLEMENTED)

### Phase 6: Text Extraction and Output ✅ COMPLETED
- [x] Implement file content reading
- [x] Add text extraction logic
- [x] Create output formatting:
  - File path headers
  - Content sections
  - Metadata (file size, type, etc.)
- [ ] Add streaming output for large directories (NOT IMPLEMENTED)
- [ ] Implement progress indication (NOT IMPLEMENTED)

### Phase 7: Advanced Features
- [ ] Add configuration file support (.textingestrc)
- [ ] Implement custom file type filters
- [ ] Add exclude patterns (like .gitignore)
- [ ] Create preset configurations
- [ ] Add statistics display (files count, total size)
- [ ] Implement multi-selection modes

### Phase 8: Error Handling and Edge Cases
- [ ] Add comprehensive error handling
- [ ] Handle permission denied scenarios
- [ ] Manage binary file detection and skipping
- [ ] Add large file warnings
- [ ] Implement graceful interrupt handling (Ctrl+C)

### Phase 9: Performance Optimization
- [ ] Optimize directory traversal for large trees
- [ ] Implement lazy loading for huge directories
- [ ] Add caching for repeated operations
- [ ] Optimize fuzzy matching performance
- [ ] Memory usage optimization

### Phase 10: Testing and Documentation
- [ ] Write unit tests for core functionality
- [ ] Add integration tests
- [ ] Create performance benchmarks
- [ ] Write comprehensive README
- [ ] Add usage examples and screenshots
- [ ] Create man page documentation

## File Structure
```
text-ingest-cli/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── cli.rs              # Command line interface
│   ├── directory/
│   │   ├── mod.rs
│   │   ├── tree.rs         # Directory tree structure
│   │   ├── traversal.rs    # File system traversal
│   │   └── state.rs        # Include/exclude state management
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── app.rs          # Main application state
│   │   ├── interface.rs    # TUI interface
│   │   ├── events.rs       # Event handling
│   │   └── colors.rs       # Color scheme
│   ├── fuzzy/
│   │   ├── mod.rs
│   │   ├── matcher.rs      # Fuzzy matching logic
│   │   └── filter.rs       # Search filtering
│   ├── output/
│   │   ├── mod.rs
│   │   ├── formatter.rs    # Output formatting
│   │   └── writer.rs       # File writing
│   └── config/
│       ├── mod.rs
│       └── settings.rs     # Configuration management
├── tests/
├── examples/
└── README.md
```

## Key Dependencies
```toml
[dependencies]
clap = { version = "4.4", features = ["derive"] }
ratatui = "0.24"
crossterm = "0.27"
fuzzy-matcher = "0.3"
walkdir = "2.4"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

## User Experience Flow
1. User runs CLI with target directory
2. Tool scans directory structure
3. Interactive TUI opens with fuzzy finder
4. User can search and navigate through directories
5. Toggle include/exclude with visual feedback
6. Hierarchical selection automatically updates children
7. User confirms selection and generates output
8. Text ingest file is created with selected content

## Success Criteria
- Fast directory traversal (< 1s for 10k files)
- Responsive TUI (< 100ms input lag)
- Intuitive keyboard navigation
- Clear visual feedback for selection state
- Accurate hierarchical state management
- Robust error handling
- Comprehensive documentation

