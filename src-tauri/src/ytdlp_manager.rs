use std::path::{Path, PathBuf};
use std::env;
use tokio::sync::OnceCell;

// Cache the YT-DLP path to avoid repeated lookups
static YTDLP_PATH_CACHE: OnceCell<Result<PathBuf, String>> = OnceCell::const_new();

pub async fn get_ytdlp_path() -> Result<PathBuf, String> {
    // Use cached path if available, otherwise initialize
    YTDLP_PATH_CACHE
        .get_or_init(|| async { find_ytdlp_path().await })
        .await
        .clone()
}

async fn find_ytdlp_path() -> Result<PathBuf, String> {
    // First, check system PATH for yt-dlp
    let system_path = if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };

    // Check if system yt-dlp is available
    if let Ok(path) = which::which(system_path) {
        // Verify system version is up to date
        if let Ok(is_up_to_date) = check_system_ytdlp_version(&path).await {
            if is_up_to_date {
                return Ok(path);
            }
            // System version is outdated, fall through to bundled
        }
        // If version check fails, fall through to bundled as well
    }

    // Fallback to bundled YT-DLP in resources directory
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Build resource paths in order of likelihood (most likely first)
            let mut resource_paths = Vec::new();

            // On macOS, prioritize .app bundle paths
            #[cfg(target_os = "macos")]
            {
                let exe_str = exe_dir.to_string_lossy();
                if exe_str.contains("Contents/MacOS") {
                    // We're in a .app bundle - this is the most likely path
                    resource_paths.push(exe_dir.join("../Resources/resources"));
                } else {
                    // Not in bundle, try standard macOS resource location
                    resource_paths.push(exe_dir.join("../Resources/resources"));
                }
            }

            // Add other platform-specific paths
            #[cfg(not(target_os = "macos"))]
            {
                resource_paths.push(exe_dir.join("resources"));
                resource_paths.push(exe_dir.join("../resources"));
            }

            // Add fallback paths
            resource_paths.push(exe_dir.join("../../resources"));
            resource_paths.push(PathBuf::from("src-tauri/resources"));

            // Check paths in order
            for resource_dir in resource_paths {
                let bundled_path = get_platform_specific_path(&resource_dir);
                if bundled_path.exists() {
                    return Ok(bundled_path);
                }
            }
        }
    }

    Err("YT-DLP not found. Please ensure YT-DLP is installed or bundled with the application.".to_string())
}

/// Check if system yt-dlp version is up to date
/// Returns Ok(true) if up to date, Ok(false) if outdated, Err on error
async fn check_system_ytdlp_version(ytdlp_path: &PathBuf) -> Result<bool, String> {
    use crate::updater;
    
    // Get system version with timeout (5 seconds)
    let version_output = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::process::Command::new(ytdlp_path)
            .arg("--version")
            .output()
    )
    .await
    .map_err(|_| "Version check timed out".to_string())?
    .map_err(|e| format!("Failed to execute YT-DLP: {}", e))?;

    if !version_output.status.success() {
        return Err("Failed to get system YT-DLP version".to_string());
    }

    let system_version = String::from_utf8(version_output.stdout)
        .map_err(|e| format!("Failed to parse version: {}", e))?;
    let system_version = system_version.trim();

    // Get latest version from GitHub (uses cache)
    let latest_version = updater::get_latest_version().await?;

    // Compare versions
    updater::compare_ytdlp_versions(system_version, &latest_version)
        .map_err(|e| format!("Version comparison failed: {}", e))
}

fn get_platform_specific_path(resource_dir: &Path) -> PathBuf {
    let mut path = resource_dir.to_path_buf();
    
    if cfg!(target_os = "windows") {
        path.push("yt-dlp.exe");
    } else if cfg!(target_os = "macos") {
        // macOS binary is universal (works for both Intel and Apple Silicon)
        path.push("yt-dlp_macos");
    } else {
        // Linux
        if cfg!(target_arch = "aarch64") {
            path.push("yt-dlp_linux_arm64");
        } else {
            path.push("yt-dlp_linux");
        }
    }
    
    path
}

/// Determine if the YT-DLP path is from system PATH or bundled
pub async fn get_ytdlp_source(ytdlp_path: &PathBuf) -> String {
    // Check if the path is in system PATH
    let system_path = if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };
    
    // Check if we can find it in PATH and if it matches our path
    if let Ok(path_in_path) = which::which(system_path) {
        // Try to canonicalize both paths for comparison
        let canonical_path = path_in_path.canonicalize().ok();
        let canonical_ytdlp = ytdlp_path.canonicalize().ok();
        
        // Check if paths match (either exact match or canonicalized match)
        if path_in_path == *ytdlp_path || 
           (canonical_path.is_some() && canonical_path == canonical_ytdlp) {
            return "path".to_string();
        }
    }
    
    // If not found in PATH, it's bundled
    // Bundled paths typically contain "resources" or have platform-specific names
    "bundled".to_string()
}

pub fn get_bundled_ytdlp_dir() -> Result<PathBuf, String> {
    // Try multiple possible resource directory locations
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            #[allow(unused_mut)] // Only mutated on macOS
            let mut resource_paths = vec![
                // macOS .app bundle structure: Contents/MacOS/executable -> Contents/Resources/resources
                exe_dir.join("../Resources/resources"),
                // Windows/Linux: resources next to executable
                exe_dir.join("resources"),
                exe_dir.join("../resources"),
                exe_dir.join("../../resources"),
                PathBuf::from("src-tauri/resources"),
            ];

            // On macOS, also try additional paths for .app bundles
            #[cfg(target_os = "macos")]
            {
                // Check if we're in a .app bundle by looking for Contents/MacOS in the path
                let exe_str = exe_dir.to_string_lossy();
                if exe_str.contains("Contents/MacOS") {
                    // We're in a .app bundle, resources are at Contents/Resources/resources
                    resource_paths.insert(0, exe_dir.join("../../Resources/resources"));
                }
                // Also try the standard macOS resource location (relative to MacOS dir)
                resource_paths.insert(0, exe_dir.join("../Resources/resources"));
            }

            for resource_dir in resource_paths {
                if resource_dir.exists() {
                    return Ok(resource_dir);
                }
            }
        }
    }

    // Fallback: create resources directory in current directory
    let fallback_dir = PathBuf::from("src-tauri/resources");
    std::fs::create_dir_all(&fallback_dir)
        .map_err(|e| format!("Failed to create resource directory: {}", e))?;
    Ok(fallback_dir)
}

