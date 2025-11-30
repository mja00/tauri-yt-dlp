#!/bin/bash
# Bash script to download YT-DLP binaries for all platforms
# This script downloads the latest YT-DLP releases for Windows, macOS (Intel and ARM), and Linux (x64 and ARM64)

RESOURCES_DIR="src-tauri/resources"
mkdir -p "$RESOURCES_DIR"

echo "Fetching latest YT-DLP release information..."

RELEASE_URL="https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest"
RELEASE_INFO=$(curl -s -H "User-Agent: mac-ytdlp-downloader" "$RELEASE_URL")

VERSION=$(echo "$RELEASE_INFO" | grep -o '"tag_name": "[^"]*' | grep -o '[^"]*$')
echo "Latest version: $VERSION"

# Function to find and download asset
find_and_download() {
    local patterns=("$@")
    local output_name=${patterns[-1]}
    unset 'patterns[${#patterns[@]}-1]'
    
    local url=""
    for pattern in "${patterns[@]}"; do
        url=$(echo "$RELEASE_INFO" | grep -o "\"browser_download_url\": \"[^\"]*$pattern[^\"]*\"" | grep -o 'https://[^"]*' | head -1)
        if [ -n "$url" ]; then
            break
        fi
    done
    
    if [ -n "$url" ]; then
        echo "Downloading $output_name..."
        curl -L -o "$RESOURCES_DIR/$output_name" "$url"
        chmod +x "$RESOURCES_DIR/$output_name"
        echo "$output_name downloaded"
        return 0
    else
        echo "Warning: Could not find asset for $output_name"
        return 1
    fi
}

# Download Windows binary
find_and_download "yt-dlp.exe" "yt-dlp.exe"

# Download macOS binary (universal - works for both Intel and Apple Silicon)
find_and_download "macos" "yt-dlp_macos" "yt-dlp_macos"

# Download Linux binaries  
find_and_download "linux" "yt-dlp_linux" "yt-dlp_linux"
find_and_download "linux.*arm64" "linux.*aarch64" "yt-dlp_linux_arm64"

echo ""
echo "All binaries downloaded to $RESOURCES_DIR"

