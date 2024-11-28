use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use localsend_protocol::{
    model::{FileInfo, FileRequest, UploadParam},
    request::{prepare_upload, upload},
    server::OutMessage,
};
use tauri::Emitter;

use crate::model::AppState;

#[tauri::command(async)]
pub async fn get_device_info(app_state: tauri::State<'_, AppState>) -> Result<String, String> {
    let device = app_state.setting.read().await.to_device_message(None);
    let ipv4s = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|iface| match iface.addr {
            if_addrs::IfAddr::V4(v4) if !v4.is_loopback() => Some(v4.ip.into()),
            _ => None,
        })
        .collect::<Vec<IpAddr>>();

    Ok(serde_json::json!((device, ipv4s)).to_string())
}

#[tauri::command(async)]
pub async fn refresh(app_state: tauri::State<'_, AppState>) -> Result<(), String> {
    match app_state.sender.read().await.as_ref() {
        Some(sender) => {
            let _ = sender.send(OutMessage::Refresh).await;
        }
        None => {
            log::error!("OutMessage Sender is None?");
        }
    }
    Ok(())
}

#[cfg(not(target_os = "android"))]
#[tauri::command]
pub async fn open_file_picker(app: tauri::AppHandle) -> Result<String, String> {
    use std::fs;

    use localsend_protocol::model::FileInfo;
    use tauri_plugin_dialog::DialogExt;

    let files = app
        .dialog()
        .file()
        .blocking_pick_files()
        .unwrap_or_default();
    let mut id_path = HashMap::new();
    let file_infos = files
        .into_iter()
        .map(|file_path| {
            let path = file_path.as_path().unwrap();
            let metadata = fs::metadata(path).unwrap();
            let id = uuid::Uuid::new_v4().to_string();
            id_path.insert(id.clone(), path.to_str().unwrap_or_default().to_string());
            FileInfo {
                id,
                file_name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                file_type: path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                size: metadata.len(),
                sha256: None,
                preview: None,
            }
        })
        .collect::<Vec<FileInfo>>();
    Ok(serde_json::json!([id_path, file_infos]).to_string())
}

#[cfg(target_os = "android")]
#[tauri::command]
pub async fn open_file_picker(app: tauri::AppHandle) -> Result<String, String> {
    use file_picker_android::PickerPlugin;
    use tauri::Manager;
    use std::fs;

    let picker_plugin = app.state::<PickerPlugin<tauri::Wry>>();
    let files = picker_plugin.pick_files().unwrap_or(Vec::new());
    let mut id_path = HashMap::new();
    let file_infos = files
        .into_iter()
        .map(|file_path| {
            let path = file_path.as_path();
            let metadata = fs::metadata(path).unwrap();
            let id = uuid::Uuid::new_v4().to_string();
            id_path.insert(id.clone(), path.to_str().unwrap_or_default().to_string());
            FileInfo {
                id,
                file_name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                file_type: path
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                size: metadata.len(),
                sha256: None,
                preview: None,
            }
        })
        .collect::<Vec<FileInfo>>();
    Ok(serde_json::json!([id_path, file_infos]).to_string())
}

#[tauri::command(async)]
pub async fn prepare_upload_files(
    app_state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
    id_path: HashMap<String, String>,
    file_infos: Vec<FileInfo>,
    addr: String,
    port: u16,
) -> Result<(), String> {
    if let Err(e) = app.emit("upload", file_infos.clone()) {
        log::error!("emit error: {e:?}");
    }
    let addr: SocketAddr = addr.parse().unwrap();
    let addr = SocketAddr::new(addr.ip(), port);
    let info = app_state.setting.read().await.to_device_message(None);
    let files = file_infos
        .iter()
        .map(|file_info| (file_info.id.to_owned(), file_info.clone()))
        .collect::<HashMap<String, FileInfo>>();
    let file_req = FileRequest { info, files };
    let resp = prepare_upload(file_req, &addr)
        .await
        .map_err(|e| e.to_string())?;
    let agreed_vec = resp
        .files
        .keys()
        .map(|s| s.to_owned())
        .collect::<Vec<String>>();
    if let Err(e) = app.emit("agreed-upload", agreed_vec.clone()) {
        log::error!("emit error: {e:?}");
    }
    let mut handles = vec![];
    for id in agreed_vec {
        let token = resp.files.get(&id).unwrap().clone();
        let upload_param = UploadParam {
            session_id: resp.session_id.clone(),
            file_id: id.clone(),
            token,
        };
        let file_path = PathBuf::from(id_path.get(&id).unwrap());
        let addr = addr.clone();
        let join_handle =
            tokio::spawn(async move { upload(upload_param, &file_path, &addr).await });
        handles.push(join_handle);
    }

    for handle in handles {
        match handle.await {
            Ok(res) if res.is_err() => {
                log::error!("upload error: {:?}", res.unwrap_err());
            }
            Ok(_) => {}
            Err(e) => {
                log::error!("upload error: {}", e);
            }
        }
    }
    Ok(())
}
