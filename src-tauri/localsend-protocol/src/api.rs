use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{
    body::BodyDataStream,
    extract::{ConnectInfo, Query, Request, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter},
    sync::RwLock,
};
use tokio_stream::StreamExt;

use crate::{
    mission::Mission,
    model::{DeviceMessage, FileRequest, FileResponse, UploadParam},
};

#[derive(Clone)]
pub struct AppState {
    pub devices: Arc<RwLock<HashMap<String, (SocketAddr, DeviceMessage)>>>,
    pub messions: Arc<RwLock<HashMap<String, Mission>>>,
}

pub async fn handle_register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<DeviceMessage>,
) {
    let mut devices = state.devices.write().await;
    devices.insert(payload.fingerprint.to_owned(), (addr, payload));
}

pub async fn handle_prepare_upload(
    State(state): State<AppState>,
    Json(payload): Json<FileRequest>,
) -> Result<Json<FileResponse>, StatusCode> {
    let divices = state.devices.read().await;
    let device = if let Some(device) = divices.get(&payload.info.fingerprint) {
        device.clone()
    } else {
        return Err(StatusCode::FORBIDDEN);
    };

    let mission = Mission::new(payload.files, device.1);

    // TODO: 同意判断
    let mut messions = state.messions.write().await;
    messions.insert(mission.id.clone(), mission.clone());

    let file_resp = FileResponse {
        session_id: mission.id,
        files: mission.id_token_map,
    };
    Ok(Json(file_resp))
}

pub async fn handle_upload(
    State(state): State<AppState>,
    param: Query<UploadParam>,
    request: Request,
) -> Result<(), StatusCode> {
    let param = param.0;
    let mission = state.messions.read().await;
    let mission = match mission.get(&param.session_id) {
        Some(m) => m,
        None => return Err(StatusCode::FORBIDDEN),
    };
    let store_path = "/Users/cakeal/Downloads";
    let file = mission.info_map.get(&param.file_id).unwrap();
    let body_stream = request.into_body().into_data_stream();
    save_to_file(store_path, &file.file_name, body_stream)
        .await
        .map_err(|e| {
            eprintln!("Error saving file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn save_to_file(
    dir: &str,
    file_name: &str,
    stream: BodyDataStream,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(dir).join(file_name);
    let file = File::create(file_path).await?;
    let mut writer = BufWriter::new(file);
    let mut stream =
        stream.map(|res| res.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)));
    while let Some(chunk_res) = stream.next().await {
        match chunk_res {
            Ok(chunk) => {
                writer.write_all(&chunk).await?;
            }
            Err(err) => {
                return Err(Box::new(err));
            }
        }
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct SessionId {
    #[serde(alias = "sessionId")]
    pub id: String,
}

pub async fn cancel(
    State(state): State<AppState>,
    session_id: Query<SessionId>,
) {
    // TODO:
}

// TODO: 3.2 HTTP Legacy Mode 未实现
// TODO: 5 下载 API 未实现