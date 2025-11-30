# PowerShell script to download YT-DLP binaries for all platforms
# This script downloads the latest YT-DLP releases for Windows, macOS (Intel and ARM), and Linux (x64 and ARM64)

$resourcesDir = "src-tauri\resources"
New-Item -ItemType Directory -Force -Path $resourcesDir | Out-Null

Write-Host "Fetching latest YT-DLP release information..."

$releaseUrl = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest"
$releaseInfo = Invoke-RestMethod -Uri $releaseUrl -Headers @{"User-Agent"="mac-ytdlp-downloader"}

Write-Host "Latest version: $($releaseInfo.tag_name)"

# Download Windows binary
$windowsAsset = $releaseInfo.assets | Where-Object { $_.name -eq "yt-dlp.exe" }
if ($windowsAsset) {
    Write-Host "Downloading Windows binary..."
    Invoke-WebRequest -Uri $windowsAsset.browser_download_url -OutFile "$resourcesDir\yt-dlp.exe"
    Write-Host "Windows binary downloaded"
} else {
    Write-Host "Warning: Windows binary not found in release assets"
}

# Download macOS binary (universal - works for both Intel and Apple Silicon)
# YT-DLP releases typically have: yt-dlp_macos, yt-dlp_macos_legacy, etc.
$macosAssets = $releaseInfo.assets | Where-Object { 
    $_.name -like "*macos*" -and 
    $_.name -notlike "*arm64*" -and 
    $_.name -notlike "*aarch64*" -and
    $_.name -notlike "*.sig" -and
    $_.name -notlike "*.tar.gz"
}

if ($macosAssets) {
    $macosAsset = $macosAssets | Select-Object -First 1
    Write-Host "Downloading macOS binary (universal for Intel and Apple Silicon)..."
    Invoke-WebRequest -Uri $macosAsset.browser_download_url -OutFile "$resourcesDir\yt-dlp_macos"
    Write-Host "macOS binary downloaded"
} else {
    Write-Host "Warning: macOS binary not found in release assets"
}

# Download Linux binaries
$linuxAssets = $releaseInfo.assets | Where-Object { 
    $_.name -like "*linux*" -and 
    $_.name -notlike "*arm64*" -and 
    $_.name -notlike "*aarch64*" -and
    $_.name -notlike "*.sig" -and
    $_.name -notlike "*.tar.gz"
}

if ($linuxAssets) {
    $linuxAsset = $linuxAssets | Select-Object -First 1
    Write-Host "Downloading Linux x64 binary..."
    Invoke-WebRequest -Uri $linuxAsset.browser_download_url -OutFile "$resourcesDir\yt-dlp_linux"
    Write-Host "Linux x64 binary downloaded"
} else {
    Write-Host "Warning: Linux x64 binary not found in release assets"
}

# Try to find ARM64 Linux binary
$linuxArmAssets = $releaseInfo.assets | Where-Object { 
    ($_.name -like "*linux*arm64*" -or $_.name -like "*linux*aarch64*") -and 
    $_.name -notlike "*.sig" -and
    $_.name -notlike "*.tar.gz"
}

if ($linuxArmAssets) {
    $linuxArmAsset = $linuxArmAssets | Select-Object -First 1
    Write-Host "Downloading Linux ARM64 binary..."
    Invoke-WebRequest -Uri $linuxArmAsset.browser_download_url -OutFile "$resourcesDir\yt-dlp_linux_arm64"
    Write-Host "Linux ARM64 binary downloaded"
} else {
    Write-Host "Warning: Linux ARM64 binary not found (may not be available)"
}

Write-Host "`nAll binaries downloaded to $resourcesDir"
Write-Host "Note: Make sure to make Linux/macOS binaries executable if building on those platforms"

