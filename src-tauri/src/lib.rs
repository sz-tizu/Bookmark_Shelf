pub mod checker;
pub mod config;
pub mod exporter;
pub mod importer;

use config::{Config, default_bookmark_dir};
use std::path::PathBuf;

#[tauri::command]
async fn import_bookmarks(html_path: String, dest_dir: String) -> Result<importer::ImportStats, String> {
    importer::import_html(
        &PathBuf::from(&html_path),
        &PathBuf::from(&dest_dir),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_bookmarks(src_dir: String, output_path: String) -> Result<exporter::ExportStats, String> {
    exporter::export_to_html(
        &PathBuf::from(&src_dir),
        &PathBuf::from(&output_path),
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_links(
    dir: String,
    concurrency: usize,
    timeout_secs: u64,
) -> Result<Vec<checker::CheckResult>, String> {
    checker::check_all(&PathBuf::from(&dir), concurrency, timeout_secs)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config() -> Result<Config, String> {
    Config::load().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config(config: Config) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
fn read_dir_tree(dir: String) -> Result<serde_json::Value, String> {
    dir_node(&PathBuf::from(&dir)).map_err(|e| e.to_string())
}

fn dir_node(path: &PathBuf) -> anyhow::Result<serde_json::Value> {
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| {
            let is_file = e.path().is_file();
            (is_file, e.file_name())
        });
        let children: Vec<_> = entries
            .iter()
            .filter_map(|e| dir_node(&e.path()).ok())
            .collect();
        Ok(serde_json::json!({
            "name": path.file_name().unwrap_or_default().to_string_lossy(),
            "path": path.to_string_lossy(),
            "type": "folder",
            "children": children
        }))
    } else if path.extension().and_then(|s| s.to_str()) == Some("url") {
        let title = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let url = read_url(path).unwrap_or_default();
        Ok(serde_json::json!({
            "name": title,
            "path": path.to_string_lossy(),
            "type": "bookmark",
            "url": url
        }))
    } else {
        anyhow::bail!("skip")
    }
}

fn read_url(path: &PathBuf) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    content
        .lines()
        .find(|l| l.trim().starts_with("URL="))
        .map(|l| l.trim().trim_start_matches("URL=").to_string())
}

#[tauri::command]
fn open_in_finder(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|_app| {
            // デフォルトのブックマークディレクトリを初回起動時に作成する
            let bookmark_dir = Config::load()
                .map(|c| c.general.bookmark_dir)
                .unwrap_or_else(|_| default_bookmark_dir());
            if let Err(e) = std::fs::create_dir_all(&bookmark_dir) {
                eprintln!("Warning: could not create bookmark dir {bookmark_dir}: {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            import_bookmarks,
            export_bookmarks,
            check_links,
            get_config,
            save_config,
            read_dir_tree,
            open_in_finder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
