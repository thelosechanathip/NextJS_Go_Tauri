// src-tauri/src/lib.rs

use std::sync::{Arc, Mutex};

use tauri::Manager;
use tauri_plugin_shell::{
    process::{CommandChild, CommandEvent},
    ShellExt,
};

#[derive(Default)]
struct BackendProcess(Arc<Mutex<Option<CommandChild>>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // shell plugin
        .plugin(tauri_plugin_shell::init())
        // log plugin
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        // state สำหรับจัดการ backend process
        .manage(BackendProcess::default())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // 🔹 DEV MODE: ใช้ Go + air → ไม่ต้องรัน sidecar
            if cfg!(debug_assertions) {
                log::info!("DEV MODE: ใช้ Go air ไม่รัน backend sidecar");
                return Ok(());
            }

            // 🔹 PRODUCTION MODE: รัน backend เป็น sidecar จาก externalBin
            let state = app_handle.state::<BackendProcess>().0.clone();

            tauri::async_runtime::spawn(async move {
                // หน่วงนิดหน่อยให้ UI ขึ้นก่อน
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                // IMPORTANT:
                // ใน tauri.conf.json → "externalBin": ["binaries/backend"]
                // ดังนั้นชื่อ sidecar ที่ใช้กับ shell().sidecar() คือ "backend"
                match app_handle.shell().sidecar("backend") {
                    Ok(mut command) => {
                        match command.spawn() {
                            Ok((mut rx, child)) => {
                                let pid = child.pid();
                                *state.lock().unwrap() = Some(child);
                                log::info!("Backend sidecar เริ่มแล้ว PID: {pid}");

                                // pipe stdout จาก backend → tauri log
                                tauri::async_runtime::spawn(async move {
                                    while let Some(event) = rx.recv().await {
                                        if let CommandEvent::Stdout(line) = event {
                                            log::info!(
                                                "backend → {}",
                                                String::from_utf8_lossy(&line)
                                            );
                                        }
                                    }
                                });
                            }
                            Err(e) => {
                                log::error!("spawn backend sidecar ไม่ได้: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("สร้าง sidecar(\"backend\") ไม่ได้: {e}");
                    }
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // ปิดแอปแล้ว ให้ kill backend ด้วย (เฉพาะ production)
                if !cfg!(debug_assertions) {
                    if let Ok(mut guard) = window.state::<BackendProcess>().0.lock() {
                        if let Some(mut child) = guard.take() {
                            if let Err(e) = child.kill() {
                                log::warn!("kill backend ไม่สำเร็จ: {e}");
                            } else {
                                log::info!("kill backend sidecar สำเร็จ");
                            }
                        }
                    }
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
