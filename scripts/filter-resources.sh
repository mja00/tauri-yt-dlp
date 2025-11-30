#!/bin/bash
# Script to filter YT-DLP resources to only include the current platform's binary

RESOURCES_DIR="src-tauri/resources"

# Determine which binary to keep based on the target platform
if [ "$1" = "windows" ] || [ "$1" = "x86_64-pc-windows-msvc" ]; then
    KEEP="yt-dlp.exe"
    REMOVE=("yt-dlp_macos" "yt-dlp_linux" "yt-dlp_linux_arm64")
elif [ "$1" = "macos" ] || [ "$1" = "aarch64-apple-darwin" ] || [ "$1" = "x86_64-apple-darwin" ]; then
    KEEP="yt-dlp_macos"
    REMOVE=("yt-dlp.exe" "yt-dlp_linux" "yt-dlp_linux_arm64")
elif [ "$1" = "linux" ] || [ "$1" = "x86_64-unknown-linux-gnu" ]; then
    KEEP="yt-dlp_linux"
    REMOVE=("yt-dlp.exe" "yt-dlp_macos" "yt-dlp_linux_arm64")
elif [ "$1" = "linux-arm64" ] || [ "$1" = "aarch64-unknown-linux-gnu" ]; then
    KEEP="yt-dlp_linux_arm64"
    REMOVE=("yt-dlp.exe" "yt-dlp_macos" "yt-dlp_linux")
else
    # Default: keep all (for development)
    echo "No platform specified, keeping all binaries"
    exit 0
fi

echo "Filtering resources for platform: $1"
echo "Keeping: $KEEP"

# Remove binaries for other platforms
for binary in "${REMOVE[@]}"; do
    if [ -f "$RESOURCES_DIR/$binary" ]; then
        echo "Removing: $binary"
        rm -f "$RESOURCES_DIR/$binary"
    fi
done

echo "Resource filtering complete"

