// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod ytdlp_manager;
mod updater;
mod config;

use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_video_info,
            get_ytdlp_version,
            check_ytdlp_update,
            update_ytdlp,
            get_download_location,
            set_download_location,
            get_video_formats,
            download_video,
            cancel_download,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

