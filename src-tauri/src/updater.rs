use crate::ytdlp_manager;
use std::fs;
use std::sync::OnceLock;
use std::time::SystemTime;

const YTDLP_RELEASES_API: &str = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest";

// Cache for latest version with timestamp
struct CachedVersion {
    version: String,
    cached_at: SystemTime,
}

static LATEST_VERSION_CACHE: OnceLock<CachedVersion> = OnceLock::new();
const CACHE_DURATION_SECONDS: u64 = 3600; // 1 hour

#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(serde::Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub async fn check_update_available() -> Result<bool, String> {
    let current_version = get_current_version().await?;
    let latest_version = get_latest_version().await?;

    Ok(current_version != latest_version)
}

async fn get_current_version() -> Result<String, String> {
    let ytdlp_path = ytdlp_manager::get_ytdlp_path().await?;
    
    let output = tokio::process::Command::new(&ytdlp_path)
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to get current version: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get current version".to_string());
    }

    let version = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse version: {}", e))?;

    Ok(version.trim().to_string())
}

pub async fn get_latest_version() -> Result<String, String> {
    // Check cache first
    if let Some(cached) = LATEST_VERSION_CACHE.get() {
        if let Ok(elapsed) = SystemTime::now().duration_since(cached.cached_at) {
            if elapsed.as_secs() < CACHE_DURATION_SECONDS {
                return Ok(cached.version.clone());
            }
        }
    }

    // Fetch from API
    let client = reqwest::Client::new();
    let response = client
        .get(YTDLP_RELEASES_API)
        .header("User-Agent", "mac-ytdlp-updater")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch latest version: {}", e))?;

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let version = release.tag_name.trim_start_matches('v').to_string();
    
    // Update cache
    let _ = LATEST_VERSION_CACHE.set(CachedVersion {
        version: version.clone(),
        cached_at: SystemTime::now(),
    });

    Ok(version)
}

/// Compare two yt-dlp version strings
/// Returns:
/// - Ok(true) if version1 >= version2 (version1 is up to date or newer)
/// - Ok(false) if version1 < version2 (version1 is outdated)
/// - Err if versions cannot be compared
pub fn compare_ytdlp_versions(version1: &str, version2: &str) -> Result<bool, String> {
    // Handle nightly versions: nightly@YYYY.MM.DD.HHMMSS
    let v1_is_nightly = version1.starts_with("nightly@");
    let v2_is_nightly = version2.starts_with("nightly@");
    
    // Extract base version (YYYY.MM.DD or YYYY.MM.DD.HHMMSS)
    let v1_base = if v1_is_nightly {
        version1.strip_prefix("nightly@").unwrap_or(version1)
    } else {
        version1
    };
    
    let v2_base = if v2_is_nightly {
        version2.strip_prefix("nightly@").unwrap_or(version2)
    } else {
        version2
    };
    
    // Parse version components
    let v1_parts: Vec<&str> = v1_base.split('.').collect();
    let v2_parts: Vec<&str> = v2_base.split('.').collect();
    
    // Compare year
    let v1_year: u32 = v1_parts.get(0)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Invalid year in version: {}", version1))?;
    let v2_year: u32 = v2_parts.get(0)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Invalid year in version: {}", version2))?;
    
    if v1_year != v2_year {
        return Ok(v1_year >= v2_year);
    }
    
    // Compare month
    let v1_month: u32 = v1_parts.get(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Invalid month in version: {}", version1))?;
    let v2_month: u32 = v2_parts.get(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Invalid month in version: {}", version2))?;
    
    if v1_month != v2_month {
        return Ok(v1_month >= v2_month);
    }
    
    // Compare day
    let v1_day: u32 = v1_parts.get(2)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Invalid day in version: {}", version1))?;
    let v2_day: u32 = v2_parts.get(2)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("Invalid day in version: {}", version2))?;
    
    if v1_day != v2_day {
        return Ok(v1_day >= v2_day);
    }
    
    // If both are nightly, compare time component (HHMMSS)
    if v1_is_nightly && v2_is_nightly {
        let v1_time: u32 = v1_parts.get(3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let v2_time: u32 = v2_parts.get(3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        return Ok(v1_time >= v2_time);
    }
    
    // If one is nightly and one is stable, nightly is considered newer
    if v1_is_nightly && !v2_is_nightly {
        return Ok(true);
    }
    if !v1_is_nightly && v2_is_nightly {
        return Ok(false);
    }
    
    // Same date, both stable - consider equal
    Ok(true)
}

pub async fn update_ytdlp() -> Result<String, String> {
    let resource_dir = ytdlp_manager::get_bundled_ytdlp_dir()?;
    
    // Get the appropriate asset for current platform
    let asset_name = get_platform_asset_name();
    let download_url = get_download_url(&asset_name).await?;

    // Download the new binary
    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .header("User-Agent", "mac-ytdlp-updater")
        .send()
        .await
        .map_err(|e| format!("Failed to download YT-DLP: {}", e))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;

    // Determine the target path
    let target_path = if cfg!(target_os = "windows") {
        resource_dir.join("yt-dlp.exe")
    } else if cfg!(target_os = "macos") {
        // macOS binary is universal (works for both Intel and Apple Silicon)
        resource_dir.join("yt-dlp_macos")
    } else {
        if cfg!(target_arch = "aarch64") {
            resource_dir.join("yt-dlp_linux_arm64")
        } else {
            resource_dir.join("yt-dlp_linux")
        }
    };

    // Ensure resource directory exists
    fs::create_dir_all(&resource_dir)
        .map_err(|e| format!("Failed to create resource directory: {}", e))?;

    // Write the new binary
    fs::write(&target_path, bytes)
        .map_err(|e| format!("Failed to write YT-DLP binary: {}", e))?;

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&target_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&target_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    let version = get_latest_version().await?;
    Ok(format!("Updated to version {}", version))
}

fn get_platform_asset_name() -> String {
    if cfg!(target_os = "windows") {
        "yt-dlp.exe".to_string()
    } else if cfg!(target_os = "macos") {
        // macOS binary is universal (works for both Intel and Apple Silicon)
        "yt-dlp_macos".to_string()
    } else {
        if cfg!(target_arch = "aarch64") {
            "yt-dlp_linux_arm64".to_string()
        } else {
            "yt-dlp_linux".to_string()
        }
    }
}

async fn get_download_url(asset_name: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let response = client
        .get(YTDLP_RELEASES_API)
        .header("User-Agent", "mac-ytdlp-updater")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    // First, try exact match
    for asset in &release.assets {
        if asset.name == asset_name {
            return Ok(asset.browser_download_url.clone());
        }
    }

    // Then try partial match
    for asset in &release.assets {
        if asset.name.contains(asset_name) && !asset.name.contains(".sig") {
            return Ok(asset.browser_download_url.clone());
        }
    }

    // Platform-specific fallback matching
    let platform_patterns = if cfg!(target_os = "windows") {
        vec!["yt-dlp.exe"]
    } else if cfg!(target_os = "macos") {
        // macOS binary is universal (works for both Intel and Apple Silicon)
        vec!["macos"]
    } else {
        if cfg!(target_arch = "aarch64") {
            vec!["linux", "arm64", "aarch64"]
        } else {
            vec!["linux"]
        }
    };

    // Try to find asset matching platform patterns
    for asset in &release.assets {
        if asset.name.contains("yt-dlp") 
            && !asset.name.contains(".sig")
            && !asset.name.ends_with(".tar.gz")
            && !asset.name.ends_with(".zip") {
            let name_lower = asset.name.to_lowercase();
            let matches_platform = platform_patterns.iter().any(|pattern| name_lower.contains(pattern));
            
            if matches_platform {
                // Additional check: Windows needs .exe, others shouldn't have .exe
                let is_windows_exe = cfg!(target_os = "windows") && asset.name.ends_with(".exe");
                let is_unix_binary = !cfg!(target_os = "windows") && !asset.name.ends_with(".exe");
                
                if is_windows_exe || is_unix_binary {
                    return Ok(asset.browser_download_url.clone());
                }
            }
        }
    }

    Err(format!("No suitable YT-DLP binary found for platform: {}", asset_name))
}

