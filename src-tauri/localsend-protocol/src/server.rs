use std::{
    collections::{HashMap, HashSet},
    net::{SocketAddr, SocketAddrV4},
    sync::Arc,
};

use axum::{routing::post, Router};
use tokio::sync::{mpsc, oneshot, RwLock};

use crate::{
    api::{handle_prepare_upload, handle_register, handle_upload},
    mission::Mission,
    model::{DeviceMessage, DeviceType, FileRequest, Protocol},
    multicast::{multicast_listener, multicast_message},
};

#[derive(Clone, Debug)]
pub struct ServerSetting {
    pub alias: String,
    pub device_model: Option<String>,
    pub device_type: Option<DeviceType>,
    pub protocol: Option<Protocol>,
    pub download: bool,
    pub port: u16,
    pub interface_addr: String,
    pub multicast_addr: String,
    pub store_path: String,
    pub fingerprint: String,
}

impl ServerSetting {
    pub fn to_device_message(&self, announce: Option<bool>) -> DeviceMessage {
        DeviceMessage {
            alias: self.alias.clone(),
            version: "2.1".to_string(),
            device_model: self.device_model.clone(),
            device_type: self.device_type.clone(),
            fingerprint: self.fingerprint.clone(),
            port: Some(self.port),
            protocol: self.protocol.clone(),
            download: self.download,
            announce,
        }
    }
}

impl Default for ServerSetting {
    fn default() -> Self {
        Self {
            alias: "".to_string(),
            device_model: Some("".to_string()),
            device_type: Some(DeviceType::Desktop),
            protocol: Some(Protocol::Http),
            download: false,
            port: 53317,
            interface_addr: "0.0.0.0".to_string(),
            multicast_addr: "224.0.0.167".to_string(),
            store_path: "/Users/cakeal/Downloads".to_string(),
            fingerprint: "".to_string(),
        }
    }
}

pub struct ServerState {
    setting: ServerSetting,
    devices: RwLock<HashMap<String, (SocketAddr, DeviceMessage)>>,
    misssions: RwLock<HashMap<String, Mission>>,
    sender: mpsc::Sender<ServerMessage>, // 从 Server 发出消息
    receiver: RwLock<mpsc::Receiver<OutMessage>>, // 从外部接受消息
}

pub enum ServerMessage {
    DeviceConnect(SocketAddr, DeviceMessage), // 设备连接
    FilePrepareUpload(FileRequest),           // 文件传入
}

pub enum OutMessage {
    FileAgreedUpload(HashSet<String>), // 同意文件传入的File Id Vec
}

pub enum InnerMessage {
    GetMyself(oneshot::Sender<DeviceMessage>),
    AddDevice(String, SocketAddr, DeviceMessage),
    GetDevice(String, oneshot::Sender<Option<DeviceMessage>>),
    FilePrepareUpload(FileRequest, oneshot::Sender<HashSet<String>>),
    AddMission(String, Mission),
    GetMission(String, oneshot::Sender<Option<Mission>>),
    GetStorePath(oneshot::Sender<String>),
}

pub struct Server {
    state: Arc<ServerState>,
}

#[derive(Debug, Clone)]
pub struct ServerHandle {
    inner_sender: mpsc::Sender<InnerMessage>, // Http Server 发出的内部信息
}

impl ServerHandle {
    pub async fn get_myself(&self) -> Option<DeviceMessage> {
        let (tx, rx) = oneshot::channel();
        let _ = self.inner_sender.send(InnerMessage::GetMyself(tx)).await;

        match rx.await {
            Err(_) => None,
            Ok(device) => Some(device),
        }
    }

    pub async fn insert_device(
        &self,
        fingerprint: String,
        addr: SocketAddr,
        device: DeviceMessage,
    ) {
        let _ = self
            .inner_sender
            .send(InnerMessage::AddDevice(fingerprint, addr, device))
            .await;
    }

    pub async fn get_device(&self, fingerprint: String) -> Option<DeviceMessage> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .inner_sender
            .send(InnerMessage::GetDevice(fingerprint, tx))
            .await;

        match rx.await {
            Err(_) => None,
            Ok(device) => device,
        }
    }

    pub async fn prepare_upload(&self, file_req: FileRequest) -> HashSet<String> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .inner_sender
            .send(InnerMessage::FilePrepareUpload(file_req, tx))
            .await;
        match rx.await {
            Err(_) => HashSet::new(),
            Ok(ids) => ids,
        }
    }

    pub async fn insert_mission(&self, mission_id: String, mission: Mission) {
        let _ = self
            .inner_sender
            .send(InnerMessage::AddMission(mission_id, mission))
            .await;
    }

    pub async fn get_mission(&self, mission_id: String) -> Option<Mission> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .inner_sender
            .send(InnerMessage::GetMission(mission_id, tx))
            .await;

        match rx.await {
            Err(_) => None,
            Ok(mission) => mission,
        }
    }

    pub async fn get_store_path(&self) -> String {
        let (tx, rx) = oneshot::channel();
        let _ = self.inner_sender.send(InnerMessage::GetStorePath(tx)).await;

        match rx.await {
            Err(_) => "".to_string(),
            Ok(path) => path,
        }
    }
}

impl Server {
    pub fn new(
        setting: ServerSetting,
        receiver: mpsc::Receiver<OutMessage>,
    ) -> (Self, mpsc::Receiver<ServerMessage>) {
        let (tx, rx) = mpsc::channel(8);
        (
            Self {
                state: Arc::new(ServerState {
                    sender: tx,
                    setting,
                    devices: RwLock::new(HashMap::new()),
                    misssions: RwLock::new(HashMap::new()),
                    receiver: RwLock::new(receiver),
                }),
            },
            rx,
        )
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let recv_addr = format!(
            "{}:{}",
            self.state.setting.multicast_addr, self.state.setting.port
        )
        .parse::<SocketAddrV4>()?;
        let device_message = self.state.setting.to_device_message(Some(true));

        // 发送组播消息
        multicast_message(&recv_addr, &device_message).await?;
        // let _ = tokio::spawn(async move {
        //     loop {
        //         if let Err(e) = multicast_message(&recv_addr, &device_message).await {
        //             log::error!("Error multicast sending: {}", e);
        //         }
        //         tokio::time::sleep(Duration::from_secs(5)).await;
        //     }
        // });

        // 监听组播
        let state1 = self.state.clone();
        let _ = tokio::spawn(async move {
            loop {
                let (device_message, sender_addr) = match multicast_listener(&recv_addr).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        // 一般是由于已经有在监听的设备了，终止监听
                        log::error!("Error multicast listening: {}", e);
                        return;
                    }
                };
                let mut devices = state1.devices.write().await;
                // 过滤已经存在的设备
                if !devices.contains_key(&device_message.fingerprint) {
                    devices.insert(
                        device_message.fingerprint.to_owned(),
                        (sender_addr, device_message.clone()),
                    );
                    state1
                        .sender
                        .send(ServerMessage::DeviceConnect(sender_addr, device_message))
                        .await
                        .unwrap();
                }
            }
        });

        // 监听服务器内部消息
        let (itx, mut irx) = mpsc::channel(8);
        let state = self.state.clone();
        let _ = tokio::spawn(async move {
            loop {
                match irx.recv().await {
                    Some(message) => {
                        state.handle_inner_message(message).await;
                    }
                    None => {}
                }
            }
        });

        // http_server
        let http_server = Router::new()
            .route("/api/localsend/v2/register", post(handle_register))
            .route(
                "/api/localsend/v2/prepare-upload",
                post(handle_prepare_upload),
            )
            .route("/api/localsend/v2/upload", post(handle_upload))
            .with_state(crate::api::AppState {
                handel: Arc::new(ServerHandle { inner_sender: itx }),
            });
        let addr = format!("0.0.0.0:{}", self.state.setting.port).parse::<SocketAddrV4>()?;
        let listener = tokio::net::TcpListener::bind(addr).await?;

        log::info!("Server started on {addr:?}");
        axum::serve(
            listener,
            http_server.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

        Ok(())
    }
}

impl ServerState {
    pub async fn handle_inner_message(&self, message: InnerMessage) {
        match message {
            InnerMessage::GetMyself(tx) => {
                let _ = tx.send(self.setting.to_device_message(None));
            }
            InnerMessage::AddDevice(fingerprint, addr, device) => {
                let mut devices = self.devices.write().await;
                if !devices.contains_key(&fingerprint) {
                    log::info!("register: {:?}, from: {:?}", &device, &addr);
                    // 通知外部接入设备
                    self.sender
                        .send(ServerMessage::DeviceConnect(addr, device.clone()))
                        .await
                        .unwrap();
                    devices.insert(fingerprint.clone(), (addr, device));
                }
            }
            InnerMessage::GetDevice(fingerprint, tx) => {
                let devices = self.devices.read().await;
                if let Some(device) = devices.get(&fingerprint) {
                    let _ = tx.send(Some(device.1.clone()));
                }
            }
            InnerMessage::FilePrepareUpload(file_req, tx) => {
                let _ = self
                    .sender
                    .send(ServerMessage::FilePrepareUpload(file_req))
                    .await;
                // 等待外部同意文件上传请求
                if let Some(OutMessage::FileAgreedUpload(agreed)) =
                    self.receiver.write().await.recv().await
                {
                    tx.send(agreed).unwrap();
                }
            }
            InnerMessage::AddMission(mission_id, mission) => {
                let mut missions = self.misssions.write().await;
                missions.insert(mission_id, mission);
            }
            InnerMessage::GetMission(mission_id, tx) => {
                let missions = self.misssions.read().await;
                if let Some(mission) = missions.get(&mission_id) {
                    let _ = tx.send(Some(mission.clone()));
                }
            }
            InnerMessage::GetStorePath(tx) => {
                let _ = tx.send(self.setting.store_path.clone());
            }
        }
    }
}
