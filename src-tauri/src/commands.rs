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

#[tauri::command]
pub async fn get_video_info(url: String) -> Result<VideoInfo, String> {
    let ytdlp_path = ytdlp_manager::get_ytdlp_path()
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
pub async fn get_ytdlp_version() -> Result<String, String> {
    let ytdlp_path = ytdlp_manager::get_ytdlp_path()
        .map_err(|e| format!("Failed to get YT-DLP path: {}", e))?;

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

    Ok(version.trim().to_string())
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

    // Sort by resolution (best first) - prioritize video formats
    formats.sort_by(|a, b| {
        // Extract numeric resolution for sorting
        let a_res: u32 = a.resolution.split('x').next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let b_res: u32 = b.resolution.split('x').next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        b_res.cmp(&a_res)
    });

    Ok(formats)
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
    let ytdlp_path = ytdlp_manager::get_ytdlp_path()
        .map_err(|e| format!("Failed to get YT-DLP path: {}", e))?;
    
    let download_dir = config::get_download_path()?;
    
    let mut cmd = tokio::process::Command::new(&ytdlp_path);
    cmd.arg("--output")
        .arg(format!("{}/%(title)s.%(ext)s", download_dir.to_string_lossy()))
        .arg("--newline")
        .arg("--progress")
        .arg("--no-warnings");
    
    // Add quality selector if specified
    if let Some(quality) = quality {
        if quality == "best" || quality == "worst" {
            // Pass "best" or "worst" directly to YT-DLP
            cmd.arg("-f").arg(quality);
        } else {
            // Assume it's a format ID
            cmd.arg("-f").arg(&quality);
        }
    } else {
        // Default to best quality
        cmd.arg("-f").arg("best");
    }
    
    cmd.arg(&url)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to execute YT-DLP: {}", e))?;
    
    // Create cancellation channel
    let (cancel_tx, mut cancel_rx) = oneshot::channel::<()>();
    {
        let mut sender_guard = CANCEL_SENDER.lock().map_err(|e| format!("Lock error: {}", e))?;
        *sender_guard = Some(cancel_tx);
    }
    
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    use tokio::io::{AsyncBufReadExt, BufReader};
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);
    let mut stdout_lines = stdout_reader.lines();
    let mut stderr_lines = stderr_reader.lines();
    
    // Parse progress from YT-DLP output
    // Multiple possible formats:
    // [download]  45.2% of 123.45MiB at 1.23MiB/s ETA 00:45
    // [download] 100% of 123.45MiB in 00:30
    // [download] Downloading item 1 of 2
    let progress_regex = regex::Regex::new(r"\[download\]\s+(\d+\.?\d*)%").unwrap();
    
    // Create a cancellation channel for the progress task
    let (progress_cancel_tx, mut progress_cancel_rx) = oneshot::channel::<()>();
    
    let window_clone = window.clone();
    let progress_task = tokio::spawn(async move {
        let mut last_progress = 0.0f64;
        loop {
            tokio::select! {
                _ = &mut progress_cancel_rx => {
                    // Cancellation requested - stop processing progress
                    break;
                }
                result = stdout_lines.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            if let Some(captures) = progress_regex.captures(&line) {
                                if let Ok(percent) = captures[1].parse::<f64>() {
                                    // Only emit if progress increased (handle out-of-order)
                                    if percent > last_progress {
                                        last_progress = percent;
                                        let _ = window_clone.emit("download-progress", percent);
                                    }
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
                    }
                }
                result = stderr_lines.next_line() => {
                    match result {
                        Ok(Some(line)) => {
                            // Progress can be in stderr
                            if let Some(captures) = progress_regex.captures(&line) {
                                if let Ok(percent) = captures[1].parse::<f64>() {
                                    // Only emit if progress increased (handle out-of-order)
                                    if percent > last_progress {
                                        last_progress = percent;
                                        let _ = window_clone.emit("download-progress", percent);
                                    }
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(_) => break,
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
    
    // Emit 100% progress to ensure UI shows completion
    let _ = window.emit("download-progress", 100.0f64);
    
    Ok(format!("Download completed to: {}", download_dir.to_string_lossy()))
}

