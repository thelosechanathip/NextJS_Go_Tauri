#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri_plugin_shell::ShellExt;

#[tauri::command]
async fn start_backend(app: tauri::AppHandle) -> Result<(), String> {
    let shell = app.shell();
    shell.sidecar("backend")
        .map_err(|e| e.to_string())?
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .invoke_handler(tauri::generate_handler![start_backend])
        .setup(|app| {
            // ดึง AppHandle ออกมา
            let app_handle = app.handle();
            // AppHandle implement Clone อยู่แล้ว
            let cloned = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let _ = start_backend(cloned).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
