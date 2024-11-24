use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    body::BodyDataStream,
    extract::{ConnectInfo, Query, Request, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use tokio::{
    fs::File,
    io::{AsyncWriteExt, BufWriter}, sync::watch, time,
};
use tokio_stream::StreamExt;

use crate::{
    mission::Mission,
    model::{DeviceMessage, FileInfo, FileRequest, FileResponse, UploadParam},
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
) -> Result<Json<DeviceMessage>, ()> {
    state
        .handel
        .insert_device(payload.fingerprint.to_owned(), addr, payload)
        .await;
    match state.handel.get_myself().await {
        Some(device) => Ok(Json(device)),
        None => Err(()),
    }
}

pub async fn handle_prepare_upload(
    State(state): State<AppState>,
    Json(payload): Json<FileRequest>,
) -> Result<Json<FileResponse>, StatusCode> {
    log::info!("prepare_upload: {:?}", &payload);
    let device = if let Some(device) = state.handel.get_device(payload.info.fingerprint.clone()).await {
        device.clone()
    } else {
        return Err(StatusCode::FORBIDDEN);
    };

    // 获取同意下载的文件 id
    let agreed_ids = state.handel.prepare_upload(payload.clone()).await;
    log::info!("{agreed_ids:?}");
    // 过滤取消传输的文件
    let files: HashMap<String, FileInfo> = payload.files.into_iter().filter(|(file_id, _)| agreed_ids.contains(file_id)).collect();
    let mission = Mission::new(files, device);
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
    let param = param.0;
    log::info!("upload: {:?}", param);
    let (file, tx) = match state.handel.get_file_info(param).await {
        Some(r) => r,
        None => return Err(StatusCode::FORBIDDEN),
    };
    let store_path = state.handel.get_store_path().await;
    let body_stream = request.into_body().into_data_stream();
    save_to_file(&store_path, &file.file_name, body_stream, tx)
        .await
        .map_err(|e| {
            log::error!("Error saving file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn save_to_file(
    dir: &str,
    file_name: &str,
    stream: BodyDataStream,
    progress: watch::Sender<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(dir).join(file_name);
    let file = File::create(file_path).await?;
    let mut writer = BufWriter::new(file);
    let mut stream =
        stream.map(|res| res.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err)));
    // 初始化定时器
    let mut interval = time::interval(Duration::from_millis(100));
    let mut total_written = 0usize;
    
    // 更新进度
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let _ = progress.send(total_written);
            }
            chunk_res = stream.next() => {
                match chunk_res {
                    Some(Ok(chunk)) => {
                        writer.write_all(&chunk).await?;
                        total_written += chunk.len();
                    }
                    Some(Err(err)) => {
                        return Err(Box::new(err));
                    }
                    None => {
                        // 完成该文件传输
                        let _ = progress.send(total_written);
                        break;
                    }
                }
            }
        }
    }
    writer.flush().await?;
    Ok(())
}

#[derive(Deserialize)]
pub struct SessionId {
    #[serde(alias = "sessionId")]
    pub id: String,
}

pub async fn handel_cancel(State(state): State<AppState>, session_id: Query<SessionId>) {
    let session_id = session_id.0.id;
    state.handel.cancel_mission(session_id).await;
}

// 3.2 HTTP Legacy Mode 未实现
// 5 下载 API 未实现
