# PowerShell script to filter YT-DLP resources to only include the current platform's binary

$resourcesDir = "src-tauri\resources"

# Determine which binary to keep based on the target platform
$platform = $args[0]

if ($platform -eq "windows" -or $platform -eq "x86_64-pc-windows-msvc") {
    $keep = "yt-dlp.exe"
    $remove = @("yt-dlp_macos", "yt-dlp_linux", "yt-dlp_linux_arm64")
}
elseif ($platform -eq "macos" -or $platform -eq "aarch64-apple-darwin" -or $platform -eq "x86_64-apple-darwin") {
    $keep = "yt-dlp_macos"
    $remove = @("yt-dlp.exe", "yt-dlp_linux", "yt-dlp_linux_arm64")
}
elseif ($platform -eq "linux" -or $platform -eq "x86_64-unknown-linux-gnu") {
    $keep = "yt-dlp_linux"
    $remove = @("yt-dlp.exe", "yt-dlp_macos", "yt-dlp_linux_arm64")
}
elseif ($platform -eq "linux-arm64" -or $platform -eq "aarch64-unknown-linux-gnu") {
    $keep = "yt-dlp_linux_arm64"
    $remove = @("yt-dlp.exe", "yt-dlp_macos", "yt-dlp_linux")
}
else {
    # Default: keep all (for development)
    Write-Host "No platform specified, keeping all binaries"
    exit 0
}

Write-Host "Filtering resources for platform: $platform"
Write-Host "Keeping: $keep"

# Remove binaries for other platforms
foreach ($binary in $remove) {
    $binaryPath = Join-Path $resourcesDir $binary
    if (Test-Path $binaryPath) {
        Write-Host "Removing: $binary"
        Remove-Item -Force $binaryPath
    }
}

Write-Host "Resource filtering complete"

