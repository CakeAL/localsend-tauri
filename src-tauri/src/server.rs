use std::collections::HashSet;

use crate::model::AppState;
use localsend_protocol::server::{Server, ServerMessage};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tokio::sync::mpsc;

pub async fn run_server(app_handle: AppHandle) {
    let (out_tx, out_rx) = mpsc::channel(8);
    let app_state = app_handle.state::<AppState>();
    let setting = app_state.setting.read().await.clone();
    *app_state.sender.write().await = Some(out_tx);
    let (server, mut server_rx) = Server::new(setting, out_rx);
    tokio::spawn(async move {
        loop {
            let app_handle = app_handle.clone();
            if let Some(message) = server_rx.recv().await {
                handle_server_message(message, app_handle).await;
            } else {
                break;
            }
        }
    });
    if let Err(e) = server.start().await {
        log::error!("server error: {e}");
    }
}

async fn handle_server_message(m: ServerMessage, app_handle: AppHandle) {
    let app_state = app_handle.state::<AppState>();
    match m {
        ServerMessage::DeviceConnect(addr, device) => {
            if let Err(e) = app_handle.emit("device-connect", (addr, &device)) {
                log::error!("emit error: {e:?}");
            }
            app_state
                .devices
                .write()
                .await
                .insert(device.fingerprint.clone(), (addr, device));
        }
        ServerMessage::FilePrepareUpload(file_req, agreed_tx) => {
            if let Err(e) = app_handle.emit("file-prepare-upload", file_req) {
                log::error!("emit error: {e:?}");
            }
            app_handle.once_any("agreed-set", |event| {
                // 你怎么敢假定前端传过来是错误的😠
                let agreed_set: HashSet<String> = serde_json::from_str(event.payload()).unwrap();
                let _ = agreed_tx.send(agreed_set);
            });
        }
        ServerMessage::Progress(file_id, mut rx) => {
            let app_handle = app_handle.clone();
            tokio::spawn(async move {
                while rx.changed().await.is_ok() {
                    // println!("file_id: {file_id}, rx: {}", *rx.borrow());
                    if let Err(e) = app_handle.emit("progress", (file_id.clone(), *rx.borrow())) {
                        log::error!("emit error: {e:?}");
                    }
                }
                // println!("file_id: {file_id}, finished");
            });
        }
        ServerMessage::CancelMission(_mission_id) => {
            // TODO:
        }
    }
}
