use crate::ytdlp_manager;
use crate::updater;
use crate::config;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Emitter;
use tokio::sync::oneshot;

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub duration: Option<u64>,
    pub uploader: Option<String>,
    pub view_count: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoFormat {
    pub format_id: String,
    pub resolution: String,
    pub ext: String,
    pub filesize: Option<u64>,
    pub quality_label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct YtdlpVersionInfo {
    pub version: String,
    pub source: String, // "path" or "bundled"
}

#[tauri::command]
pub async fn get_video_info(url: String) -> Result<VideoInfo, String> {
    let ytdlp_path = ytdlp_manager::get_ytdlp_path()
        .await
        .map_err(|e| format!("Failed to get YT-DLP path: {}", e))?;

    let output = tokio::process::Command::new(&ytdlp_path)
        .arg("--dump-json")
        .arg("--no-download")
        .arg("--no-warnings")
        .arg(&url)
        .output()
        .await
        .map_err(|e| format!("Failed to execute YT-DLP: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("YT-DLP error: {}", error_msg));
    }

    let json_output = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse YT-DLP output: {}", e))?;

    let info: serde_json::Value = serde_json::from_str(&json_output)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(VideoInfo {
        title: info["title"]
            .as_str()
            .unwrap_or("Unknown Title")
            .to_string(),
        duration: info["duration"].as_u64(),
        uploader: info["uploader"].as_str().map(|s| s.to_string()),
        view_count: info["view_count"].as_u64(),
    })
}

#[tauri::command]
pub async fn get_ytdlp_version() -> Result<YtdlpVersionInfo, String> {
    let ytdlp_path = ytdlp_manager::get_ytdlp_path()
        .await
        .map_err(|e| format!("Failed to get YT-DLP path: {}", e))?;

    // Check if the path is from system PATH or bundled
    let source = ytdlp_manager::get_ytdlp_source(&ytdlp_path).await;

    let output = tokio::process::Command::new(&ytdlp_path)
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to execute YT-DLP: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get YT-DLP version".to_string());
    }

    let version = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse version: {}", e))?;

    Ok(YtdlpVersionInfo {
        version: version.trim().to_string(),
        source,
    })
}

#[tauri::command]
pub async fn check_ytdlp_update() -> Result<bool, String> {
    updater::check_update_available().await
}

#[tauri::command]
pub async fn update_ytdlp() -> Result<String, String> {
    updater::update_ytdlp().await
}

#[tauri::command]
pub async fn get_download_location() -> Result<String, String> {
    let path = config::get_download_path()?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn set_download_location(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    
    if !path_buf.exists() {
        return Err("Path does not exist".to_string());
    }
    
    if !path_buf.is_dir() {
        return Err("Path is not a directory".to_string());
    }
    
    let mut app_config = config::load_config();
    app_config.download_location = Some(path);
    config::save_config(&app_config)?;
    
    Ok(())
}

#[tauri::command]
pub async fn get_video_formats(url: String) -> Result<Vec<VideoFormat>, String> {
    let ytdlp_path = ytdlp_manager::get_ytdlp_path()
        .await
        .map_err(|e| format!("Failed to get YT-DLP path: {}", e))?;

    // Use -J to get JSON with formats
    let output = tokio::process::Command::new(&ytdlp_path)
        .arg("-J")
        .arg("--no-warnings")
        .arg(&url)
        .output()
        .await
        .map_err(|e| format!("Failed to execute YT-DLP: {}", e))?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("YT-DLP error: {}", error_msg));
    }

    let json_output = String::from_utf8(output.stdout)
        .map_err(|e| format!("Failed to parse YT-DLP output: {}", e))?;

    let info: serde_json::Value = serde_json::from_str(&json_output)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut formats = Vec::new();

    // Parse formats from JSON
    if let Some(formats_array) = info["formats"].as_array() {
        for format in formats_array {
            // Skip audio-only formats
            if format["vcodec"].as_str() == Some("none") {
                continue;
            }
            
            let format_id = format["format_id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            
            let ext = format["ext"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            
            // Only show MP4 formats in the dropdown for QuickTime compatibility
            // Skip all other formats (webm, mkv, etc.)
            if ext != "mp4" {
                continue;
            }
            
            let resolution = if let Some(res) = format["resolution"].as_str() {
                res.to_string()
            } else {
                // Try to construct from width/height
                let width = format["width"].as_u64().unwrap_or(0);
                let height = format["height"].as_u64().unwrap_or(0);
                if width > 0 && height > 0 {
                    format!("{}x{}", width, height)
                } else {
                    "unknown".to_string()
                }
            };
            
            let filesize = format["filesize"]
                .as_u64()
                .or_else(|| format["filesize_approx"].as_u64());
            
            // Get fps if available
            let fps = format["fps"].as_f64();
            let fps_str = if let Some(fps_val) = fps {
                format!(" @ {}fps", fps_val as u32)
            } else {
                String::new()
            };
            
            // Create quality label
            let quality_label = if resolution != "unknown" {
                format!("{} ({}){}", resolution, ext.to_uppercase(), fps_str)
            } else {
                format!("Format {} ({})", format_id, ext.to_uppercase())
            };

            formats.push(VideoFormat {
                format_id,
                resolution,
                ext,
                filesize,
                quality_label,
            });
        }
    }

    // Deduplicate formats based on resolution only
    // Keep the format with the best quality (largest filesize) for each unique resolution
    // yt-dlp will automatically pair the selected video with the best available audio
    use std::collections::HashMap;
    let mut format_map: HashMap<String, VideoFormat> = HashMap::new();
    
    for format in formats {
        // Use resolution as the unique key
        let key = format.resolution.clone();
        
        // If we haven't seen this resolution, add it
        // If we have, keep the one with larger filesize (better quality)
        match format_map.get_mut(&key) {
            Some(existing) => {
                // Compare filesizes - keep the larger one
                let existing_size = existing.filesize.unwrap_or(0);
                let new_size = format.filesize.unwrap_or(0);
                if new_size > existing_size {
                    *existing = format;
                }
            }
            None => {
                format_map.insert(key, format);
            }
        }
    }
    
    // Convert back to vector
    let mut deduped_formats: Vec<VideoFormat> = format_map.into_values().collect();
    
    // Sort by resolution (best first) - all formats are MP4 for QuickTime compatibility
    deduped_formats.sort_by(|a, b| {
        // Extract numeric resolution for sorting
        let a_res: u32 = a.resolution.split('x').next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let b_res: u32 = b.resolution.split('x').next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        b_res.cmp(&a_res)
    });

    Ok(deduped_formats)
}

// Global state to store the cancel sender for cancellation
static CANCEL_SENDER: Mutex<Option<oneshot::Sender<()>>> = Mutex::new(None);

#[tauri::command]
pub async fn cancel_download() -> Result<(), String> {
    let mut sender_guard = CANCEL_SENDER.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some(sender) = sender_guard.take() {
        let _ = sender.send(());
    }
    Ok(())
}

#[tauri::command]
pub async fn download_video(url: String, quality: Option<String>, window: tauri::Window) -> Result<String, String> {
    #[cfg(debug_assertions)]
    eprintln!("[DEBUG] download_video called with url: {}, quality: {:?}", url, quality);
    
    let ytdlp_path = ytdlp_manager::get_ytdlp_path()
        .await
        .map_err(|e| {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Failed to get YT-DLP path: {}", e);
            format!("Failed to get YT-DLP path: {}", e)
        })?;
    
    #[cfg(debug_assertions)]
    eprintln!("[DEBUG] YT-DLP path: {:?}", ytdlp_path);
    
    let download_dir = config::get_download_path()
        .map_err(|e| {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Failed to get download path: {}", e);
            e
        })?;
    
    #[cfg(debug_assertions)]
    eprintln!("[DEBUG] Download directory: {:?}", download_dir);
    
    let mut cmd = tokio::process::Command::new(&ytdlp_path);
    cmd.arg("--output")
        .arg(format!("{}/%(title)s.%(ext)s", download_dir.to_string_lossy()))
        .arg("--newline")
        .arg("--progress")
        .arg("--no-warnings")
        // Force MP4 output format for QuickTime compatibility
        // YT-DLP will use native muxer for MP4 (no FFmpeg required)
        .arg("--merge-output-format")
        .arg("mp4");
    
    // Add quality selector if specified
    // Format selectors only use MP4 formats that can be merged natively without FFmpeg
    // YT-DLP's native muxer can merge MP4 video + MP4 audio streams
    if let Some(quality) = quality {
        if quality == "best" || quality == "worst" {
            // For best/worst, prefer H.264+AAC MP4 for QuickTime compatibility
            // Only select formats that are already MP4 (no re-encoding needed)
            // Format selector: prefer H.264 video + AAC audio in MP4, fallback to any MP4
            let format_selector = if quality == "best" {
                "bestvideo[ext=mp4][vcodec^=avc1]+bestaudio[ext=mp4][acodec^=mp4a]/bestvideo[ext=mp4]+bestaudio[ext=mp4]/best[ext=mp4]"
            } else {
                "worstvideo[ext=mp4][vcodec^=avc1]+worstaudio[ext=mp4][acodec^=mp4a]/worstvideo[ext=mp4]+worstaudio[ext=mp4]/worst[ext=mp4]"
            };
            cmd.arg("-f").arg(format_selector);
        } else {
            // For specific format ID (resolution), pair it with best MP4 audio
            // Only use MP4 formats to avoid needing FFmpeg
            cmd.arg("-f").arg(format!("{}+bestaudio[ext=mp4]/best[ext=mp4]", quality));
        }
    } else {
        // Default: prefer H.264+AAC MP4 for QuickTime compatibility
        // Only select MP4 formats that can be merged natively
        cmd.arg("-f").arg("bestvideo[ext=mp4][vcodec^=avc1]+bestaudio[ext=mp4][acodec^=mp4a]/bestvideo[ext=mp4]+bestaudio[ext=mp4]/best[ext=mp4]");
    }
    
    cmd.arg(&url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    
    #[cfg(debug_assertions)]
    eprintln!("[DEBUG] Executing YT-DLP command...");
    
    let mut child = cmd.spawn()
        .map_err(|e| {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Failed to spawn YT-DLP process: {}", e);
            format!("Failed to execute YT-DLP: {}", e)
        })?;
    
    #[cfg(debug_assertions)]
    eprintln!("[DEBUG] YT-DLP process spawned successfully");
    
    // Create cancellation channel
    let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
    {
        let mut sender_guard = CANCEL_SENDER.lock().map_err(|e| format!("Lock error: {}", e))?;
        *sender_guard = Some(cancel_tx);
    }
    
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let mut stdout_reader = BufReader::new(stdout);
    let mut stderr_reader = BufReader::new(stderr);
    
    // Capture console output from YT-DLP
    // Create a cancellation channel for the progress task
    let (progress_cancel_tx, mut progress_cancel_rx) = oneshot::channel::<()>();
    
    let window_clone = window.clone();
    let progress_task = tokio::spawn(async move {
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();
        
        loop {
            tokio::select! {
                _ = &mut progress_cancel_rx => {
                    // Cancellation requested - stop processing output
                    break;
                }
                result = stdout_reader.read_until(b'\n', &mut stdout_buf) => {
                    match result {
                        Ok(0) => {
                            // EOF
                            #[cfg(debug_assertions)]
                            eprintln!("[DEBUG] stdout stream ended");
                            break;
                        }
                        Ok(_) => {
                            // Convert bytes to string using lossy UTF-8 conversion
                            // This handles progress indicators and special characters gracefully
                            let line = String::from_utf8_lossy(&stdout_buf);
                            let line = line.trim_end_matches('\n').trim_end_matches('\r');
                            
                            if !line.is_empty() {
                                #[cfg(debug_assertions)]
                                eprintln!("[DEBUG] YT-DLP stdout: {}", line);
                                // Emit the line to frontend
                                let _ = window_clone.emit("download-output", line.to_string());
                            }
                            
                            stdout_buf.clear();
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[DEBUG] stdout read error: {:?}", e);
                            // Continue reading on errors - don't break the download
                            if e.kind() != std::io::ErrorKind::Interrupted {
                                break;
                            }
                        }
                    }
                }
                result = stderr_reader.read_until(b'\n', &mut stderr_buf) => {
                    match result {
                        Ok(0) => {
                            // EOF
                            #[cfg(debug_assertions)]
                            eprintln!("[DEBUG] stderr stream ended");
                            break;
                        }
                        Ok(_) => {
                            // Convert bytes to string using lossy UTF-8 conversion
                            // This handles progress indicators and special characters gracefully
                            let line = String::from_utf8_lossy(&stderr_buf);
                            let line = line.trim_end_matches('\n').trim_end_matches('\r');
                            
                            if !line.is_empty() {
                                #[cfg(debug_assertions)]
                                eprintln!("[DEBUG] YT-DLP stderr: {}", line);
                                // Emit the line to frontend (YT-DLP often uses stderr for progress)
                                let _ = window_clone.emit("download-output", line.to_string());
                            }
                            
                            stderr_buf.clear();
                        }
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[DEBUG] stderr read error: {:?}", e);
                            // Continue reading on errors - don't break the download
                            if e.kind() != std::io::ErrorKind::Interrupted {
                                break;
                            }
                        }
                    }
                }
            }
        }
    });
    
    // Wait for process or cancellation
    let status = tokio::select! {
        result = child.wait() => {
            result.map_err(|e| format!("Failed to wait for process: {}", e))?
        }
        _ = &mut cancel_rx => {
            // Cancellation requested - kill the process
            let _ = child.kill().await;
            let _ = child.wait().await;
            // Clear the cancel sender
            {
                let mut sender_guard = CANCEL_SENDER.lock().map_err(|e| format!("Lock error: {}", e))?;
                *sender_guard = None;
            }
            // Cancel progress task
            let _ = progress_cancel_tx.send(());
            let _ = progress_task.await;
            return Err("Download cancelled".to_string());
        }
    };
    
    // Cancel progress task and wait for it to finish
    let _ = progress_cancel_tx.send(());
    let _ = progress_task.await;
    
    // Clear the cancel sender
    {
        let mut sender_guard = CANCEL_SENDER.lock().map_err(|e| format!("Lock error: {}", e))?;
        *sender_guard = None;
    }
    
    if !status.success() {
        return Err("Download failed".to_string());
    }
    
    // Emit completion message
    let _ = window.emit("download-output", "Download completed successfully".to_string());
    
    Ok(format!("Download completed to: {}", download_dir.to_string_lossy()))
}

