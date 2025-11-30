use std::path::{Path, PathBuf};
use std::env;

pub fn get_ytdlp_path() -> Result<PathBuf, String> {
    // First, try to find bundled YT-DLP in resources directory
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // In development, resources might be in src-tauri/resources
            // In production, resources are bundled with the app
            let resource_paths = vec![
                exe_dir.join("resources"),
                exe_dir.join("../resources"),
                exe_dir.join("../../resources"),
                // For development
                PathBuf::from("src-tauri/resources"),
            ];

            for resource_dir in resource_paths {
                let bundled_path = get_platform_specific_path(&resource_dir);
                if bundled_path.exists() {
                    return Ok(bundled_path);
                }
            }
        }
    }

    // Fallback to system YT-DLP if bundled version not found
    let system_path = if cfg!(target_os = "windows") {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };

    // Check if system yt-dlp is available
    if which::which(system_path).is_ok() {
        return Ok(PathBuf::from(system_path));
    }

    Err("YT-DLP not found. Please ensure YT-DLP is installed or bundled with the application.".to_string())
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

pub fn get_bundled_ytdlp_dir() -> Result<PathBuf, String> {
    // Try multiple possible resource directory locations
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resource_paths = vec![
                exe_dir.join("resources"),
                exe_dir.join("../resources"),
                exe_dir.join("../../resources"),
                PathBuf::from("src-tauri/resources"),
            ];

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

