# YT-DLP Cross-Platform GUI

A native cross-platform desktop application built with Tauri that provides a simple GUI for YT-DLP. The app automatically fetches and displays video information when you enter a YouTube URL.

## Features

- 🎯 Simple, clean interface
- 🔄 Automatic video info lookup when URL is entered
- 📦 Bundled YT-DLP (no installation required)
- 🔄 Built-in YT-DLP updater
- 🍎 Native support for macOS (including Apple Silicon)
- 🪟 Windows support
- 🐧 Linux support

## Prerequisites

- Node.js (v18 or later)
- Rust (latest stable)
- System dependencies for Tauri:
  - **Windows**: Microsoft Visual Studio C++ Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`

## Icons

The application requires icon files for building. Place the following icon files in `src-tauri/icons/`:
- `icon.ico` (Windows)
- `icon.icns` (macOS)
- `icon.png` (Linux)
- Various sizes: `32x32.png`, `128x128.png`, `128x128@2x.png`

You can generate icons using online tools or image conversion utilities. For development, Tauri may work with placeholder icons, but proper icons are required for production builds.

## Setup

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd mac-ytdlp
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Download YT-DLP binaries:**
   
   On Windows (PowerShell):
   ```powershell
   .\scripts\download-ytdlp.ps1
   ```
   
   On macOS/Linux:
   ```bash
   chmod +x scripts/download-ytdlp.sh
   ./scripts/download-ytdlp.sh
   ```

   This will download the latest YT-DLP binaries for all platforms to `src-tauri/resources/`.

4. **Run the development server:**
   ```bash
   npm run tauri dev
   ```

## Building

### Development Build

```bash
npm run tauri build
```

### Production Build

The build process will create platform-specific installers in `src-tauri/target/release/bundle/`.

- **Windows**: Creates an `.msi` installer
- **macOS**: Creates a `.dmg` and `.app` bundle
- **Linux**: Creates `.deb` and `.AppImage` packages

### Platform-Specific Builds

To build for a specific platform:

```bash
# Windows
npm run tauri build -- --target x86_64-pc-windows-msvc

# macOS (Intel)
npm run tauri build -- --target x86_64-apple-darwin

# macOS (Apple Silicon)
npm run tauri build -- --target aarch64-apple-darwin

# Linux (x64)
npm run tauri build -- --target x86_64-unknown-linux-gnu

# Linux (ARM64)
npm run tauri build -- --target aarch64-unknown-linux-gnu
```

## Project Structure

```
mac-ytdlp/
├── src/                    # Frontend code
│   ├── index.html         # Main HTML
│   ├── main.js           # Frontend logic
│   └── styles.css        # Styling
├── src-tauri/            # Rust backend
│   ├── src/
│   │   ├── main.rs      # Tauri entry point
│   │   ├── commands.rs   # Tauri commands
│   │   ├── ytdlp_manager.rs  # YT-DLP path management
│   │   └── updater.rs    # YT-DLP updater
│   ├── resources/        # Bundled YT-DLP binaries
│   └── Cargo.toml        # Rust dependencies
├── scripts/              # Build/download scripts
└── package.json          # Node.js dependencies
```

## Updating YT-DLP

The app includes a built-in updater. You can check for updates and update YT-DLP from within the application, or manually run the download script to get the latest version.

## Development

### Frontend Development

The frontend uses vanilla JavaScript with Vite. Hot reload is enabled during development.

### Backend Development

The Rust backend uses Tauri v2. Commands are defined in `src-tauri/src/commands.rs`.

### Adding New Features

1. **New Tauri Command**: Add to `src-tauri/src/commands.rs` and register in `src-tauri/src/main.rs`
2. **Frontend Changes**: Modify files in `src/` directory
3. **Styling**: Update `src/styles.css`

## Troubleshooting

### YT-DLP Not Found

If you see "YT-DLP not found" errors:

1. Ensure YT-DLP binaries are in `src-tauri/resources/`
2. Run the download script to fetch binaries
3. Check that the correct binary exists for your platform

### Build Issues

- **Windows**: Ensure Visual Studio Build Tools are installed
- **macOS**: Run `xcode-select --install` if build fails
- **Linux**: Install all required system dependencies listed in Prerequisites

## License

[Add your license here]

## Contributing

[Add contributing guidelines here]
