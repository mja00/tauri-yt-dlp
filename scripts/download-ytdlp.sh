#!/bin/bash
# Bash script to download YT-DLP binaries for all platforms
# This script downloads the latest YT-DLP releases for Windows, macOS (Intel and ARM), and Linux (x64 and ARM64)

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed."
    echo "Please install jq:"
    echo "  macOS: brew install jq"
    echo "  Ubuntu/Debian: sudo apt-get install jq"
    echo "  Fedora: sudo dnf install jq"
    echo "  Arch: sudo pacman -S jq"
    exit 1
fi

RESOURCES_DIR="src-tauri/resources"
mkdir -p "$RESOURCES_DIR"

echo "Fetching latest YT-DLP release information..."

RELEASE_URL="https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest"

# Use GitHub token if provided (for CI/CD to avoid rate limits)
GITHUB_TOKEN="${GITHUB_TOKEN:-}"
if [ -n "$GITHUB_TOKEN" ]; then
    RELEASE_INFO=$(curl -s -H "User-Agent: mac-ytdlp-downloader" -H "Authorization: token $GITHUB_TOKEN" "$RELEASE_URL")
else
    RELEASE_INFO=$(curl -s -H "User-Agent: mac-ytdlp-downloader" "$RELEASE_URL")
fi

# Check if we got a valid response
if [ -z "$RELEASE_INFO" ] || echo "$RELEASE_INFO" | jq -e '.message' > /dev/null 2>&1; then
    echo "Error: Failed to fetch release information from GitHub API"
    if echo "$RELEASE_INFO" | jq -e '.message' > /dev/null 2>&1; then
        echo "API Error: $(echo "$RELEASE_INFO" | jq -r '.message')"
    fi
    exit 1
fi

VERSION=$(echo "$RELEASE_INFO" | jq -r '.tag_name')
if [ -z "$VERSION" ] || [ "$VERSION" = "null" ]; then
    echo "Error: Could not extract version from release info"
    exit 1
fi
echo "Latest version: $VERSION"

# Function to find and download asset
find_and_download() {
    local all_args=("$@")
    local num_args=${#all_args[@]}
    
    # Last argument is the output name
    local output_name=${all_args[$((num_args - 1))]}
    
    # All other arguments are patterns to search for
    local patterns=("${all_args[@]:0:$((num_args - 1))}")
    
    local url=""
    for pattern in "${patterns[@]}"; do
        # Use jq to find the asset URL matching the pattern
        url=$(echo "$RELEASE_INFO" | jq -r ".assets[] | select(.name | test(\"$pattern\"; \"i\")) | .browser_download_url" | head -1)
        if [ -n "$url" ] && [ "$url" != "null" ]; then
            break
        fi
    done
    
    if [ -n "$url" ] && [ "$url" != "null" ]; then
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

# Check if a platform filter is provided
PLATFORM_FILTER="${1:-all}"

# Download Windows binary
if [ "$PLATFORM_FILTER" = "all" ] || [ "$PLATFORM_FILTER" = "windows" ] || [ "$PLATFORM_FILTER" = "x86_64-pc-windows-msvc" ]; then
    find_and_download "yt-dlp.exe" "yt-dlp.exe"
fi

# Download macOS binary (universal - works for both Intel and Apple Silicon)
if [ "$PLATFORM_FILTER" = "all" ] || [ "$PLATFORM_FILTER" = "macos" ] || [ "$PLATFORM_FILTER" = "aarch64-apple-darwin" ] || [ "$PLATFORM_FILTER" = "x86_64-apple-darwin" ]; then
    find_and_download "macos" "yt-dlp_macos" "yt-dlp_macos"
fi

# Download Linux binaries  
if [ "$PLATFORM_FILTER" = "all" ] || [ "$PLATFORM_FILTER" = "linux" ] || [ "$PLATFORM_FILTER" = "x86_64-unknown-linux-gnu" ]; then
    find_and_download "linux" "yt-dlp_linux" "yt-dlp_linux"
fi

if [ "$PLATFORM_FILTER" = "all" ] || [ "$PLATFORM_FILTER" = "linux-arm64" ] || [ "$PLATFORM_FILTER" = "aarch64-unknown-linux-gnu" ]; then
    find_and_download "linux.*arm64" "linux.*aarch64" "yt-dlp_linux_arm64"
fi

echo ""
echo "All binaries downloaded to $RESOURCES_DIR"

