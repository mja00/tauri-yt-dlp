# Icon Setup Guide

This application requires icon files for building platform-specific installers. 

## Required Icons

Place the following icon files in `src-tauri/icons/`:

### Windows
- `icon.ico` - Windows icon file (multi-resolution)

### macOS  
- `icon.icns` - macOS icon file
- `Square107x107Logo.png` through `Square310x310Logo.png` - Various sizes for macOS
- `StoreLogo.png` - App Store logo

### Linux
- `icon.png` - PNG icon (at least 512x512 recommended)
- `32x32.png` - Small icon
- `128x128.png` - Medium icon  
- `128x128@2x.png` - High DPI icon

## Generating Icons

### Option 1: Online Tools
- Use [CloudConvert](https://cloudconvert.convcom/) or similar to convert PNG to ICO/ICNS
- Use [IconGenerator](https://www.iconfinder.com/icon-generator) or similar tools

### Option 2: Command Line (macOS/Linux)
```bash
# Convert PNG to ICO (requires ImageMagick)
convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico

# Convert PNG to ICNS (macOS only, requires iconutil)
mkdir icon.iconset
# Copy PNG files at various sizes to icon.iconset/
iconutil -c icns icon.iconset -o icon.icns
```

### Option 3: Placeholder Icons
For development/testing, you can use simple placeholder icons. The app will build but icons may not look professional.

## Quick Setup Script

You can create a simple 512x512 PNG icon and use conversion tools to generate the required formats.

