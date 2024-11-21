use std::{net::SocketAddr, path::PathBuf, sync::Arc};

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
};
use tokio_stream::StreamExt;

use crate::{
    mission::Mission,
    model::{DeviceMessage, FileRequest, FileResponse, UploadParam},
    server::ServerHandle,
};

#[derive(Clone)]
pub struct AppState {
    pub handel: Arc<ServerHandle>,
}

pub async fn handle_register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<DeviceMessage>,
) {
    log::info!("register: {:?}, from: {:?}", &payload, &addr);
    state
        .handel
        .insert_device(payload.fingerprint.to_owned(), addr, payload)
        .await;
}

pub async fn handle_prepare_upload(
    State(state): State<AppState>,
    Json(payload): Json<FileRequest>,
) -> Result<Json<FileResponse>, StatusCode> {
    log::info!("prepare_upload: {:?}", &payload);
    let device = if let Some(device) = state.handel.get_device(payload.info.fingerprint).await {
        device.clone()
    } else {
        return Err(StatusCode::FORBIDDEN);
    };

    let mission = Mission::new(payload.files, device);

    // TODO: 同意判断

    // 新建下载任务
    state
        .handel
        .insert_mission(mission.id.clone(), mission.clone())
        .await;

    let file_resp: FileResponse = FileResponse {
        session_id: mission.id,
        files: mission.id_token_map,
    };
    log::info!("agreed upload: {:?}", file_resp);
    Ok(Json(file_resp))
}

pub async fn handle_upload(
    State(state): State<AppState>,
    param: Query<UploadParam>,
    request: Request,
) -> Result<(), StatusCode> {
    log::info!("upload: {:?}", param);
    let param = param.0;
    let mission = match state.handel.get_mission(param.session_id).await {
        Some(m) => m,
        None => return Err(StatusCode::FORBIDDEN),
    };
    let store_path = state.handel.get_store_path().await;
    let file = mission.info_map.get(&param.file_id).unwrap();
    let body_stream = request.into_body().into_data_stream();
    save_to_file(&store_path, &file.file_name, body_stream)
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

pub async fn cancel(State(state): State<AppState>, session_id: Query<SessionId>) {
    // TODO:
}

// TODO: 3.2 HTTP Legacy Mode 未实现
// TODO: 5 下载 API 未实现
