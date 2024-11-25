use command::*;
use model::AppState;
use tauri::Manager;

pub mod command;
pub mod model;
pub mod server;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_device_info, refresh])
        .setup(|app| {
            let store_path = app.path().download_dir()?;
            let app_state = AppState::new(store_path);
            app.manage(app_state);
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                server::run_server(app_handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
