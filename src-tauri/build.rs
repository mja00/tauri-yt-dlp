fn main() {
    // Filter resources based on target platform before build
    filter_resources();
    tauri_build::build()
}

fn filter_resources() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    let target = env::var("TARGET").unwrap_or_default();
    let resources_dir = PathBuf::from("resources");
    
    // Only filter if we're in a build context (not development)
    // In development, we want all binaries available
    if env::var("PROFILE").unwrap_or_default() != "release" {
        return;
    }

    if !resources_dir.exists() {
        return;
    }

    // Determine which binary to keep based on target
    let (_keep, remove): (&str, &[&str]) = if target.contains("windows") {
        ("yt-dlp.exe", &["yt-dlp_macos", "yt-dlp_linux", "yt-dlp_linux_arm64"])
    } else if target.contains("apple-darwin") {
        ("yt-dlp_macos", &["yt-dlp.exe", "yt-dlp_linux", "yt-dlp_linux_arm64"])
    } else if target.contains("linux") && target.contains("aarch64") {
        ("yt-dlp_linux_arm64", &["yt-dlp.exe", "yt-dlp_macos", "yt-dlp_linux"])
    } else if target.contains("linux") {
        ("yt-dlp_linux", &["yt-dlp.exe", "yt-dlp_macos", "yt-dlp_linux_arm64"])
    } else {
        // Unknown platform, keep all
        return;
    };

    // Remove binaries for other platforms
    for binary in remove {
        let path = resources_dir.join(binary);
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
    }
}

