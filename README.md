# Mac Space Explorer

A modern disk space visualization tool built with Rust, specifically designed for macOS. This application helps you understand and manage your disk space usage through an interactive heat map visualization.

## Features

- **Interactive Heat Map Visualization**
  - Visual representation of file and directory sizes
  - Color intensity indicates relative file sizes
  - Real-time updates during scanning

- **File System Navigation**
  - Scan any directory or disk on your Mac
  - Custom path input for targeted analysis
  - Progress tracking during directory scanning

- **File Management**
  - Open files/folders directly in Finder
  - Delete files/folders from within the application
  - View detailed size information

- **Advanced Filtering**
  - Filter by file age
  - Filter by file size
  - Real-time filter updates

## Installation

### Prerequisites
- Rust (1.70 or later)
- macOS (10.15 or later)

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/chan4lk/mac-space-explorer.git
cd mac-space-explorer
```

2. Build the application:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

## Usage

1. **Starting the Application**
   - Launch the application
   - The default path is set to your home directory

2. **Scanning Directories**
   - Enter a path in the text input field
   - Click "Scan" to analyze the directory
   - Watch the progress bar for scanning status

3. **Interpreting the Heat Map**
   - Each bar represents a file or directory
   - Height indicates relative size
   - Color intensity increases with file size (red = larger files)

4. **Managing Files**
   - Click on any item to select it
   - Use "Open in Finder" to view in Finder
   - Use "Delete" to remove files/folders (use with caution)

## Project Structure

```
mac-space-explorer/
├── src/
│   ├── core/
│   │   ├── mod.rs
│   │   └── scanner.rs      # File system scanning logic
│   ├── ui/
│   │   ├── mod.rs
│   │   └── heat_map.rs     # Heat map visualization
│   └── main.rs             # Application entry point and UI
└── Cargo.toml              # Project dependencies
```

## Dependencies

- `iced`: GUI framework
- `walkdir`: Directory traversal
- `humansize`: Human-readable file sizes
- `open`: Finder integration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built with [Iced](https://github.com/iced-rs/iced) - A cross-platform GUI library for Rust
- Inspired by tools like DaisyDisk and WinDirStat
