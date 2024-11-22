use std::{collections::HashSet, net::SocketAddr};

use tokio::sync::mpsc;

use localsend_protocol::{
    model::DeviceType,
    request::send_register,
    server::{OutMessage, Server, ServerMessage, ServerSetting},
};

// for test
#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();
    let (out_tx, out_rx) = mpsc::channel(8);
    let setting = ServerSetting {
        alias: "test_device".to_string(),
        device_model: Some("test_model".to_string()),
        device_type: Some(DeviceType::Headless),
        protocol: Some(localsend_protocol::model::Protocol::Http),
        download: false,
        port: 53317,
        ..Default::default()
    };
    let (server, mut server_rx) = Server::new(setting.clone(), out_rx);
    let _ = tokio::spawn(async move {
        loop {
            if let Some(message) = server_rx.recv().await {
                match message {
                    // 服务器监听到连接请求
                    ServerMessage::DeviceConnect(addr, _device) => {
                        let addr = SocketAddr::new(addr.ip(), setting.port);
                        if let Err(e) = send_register(&setting, &addr).await {
                            log::error!("send register error: {e:?}");
                        }
                    }
                    ServerMessage::FilePrepareUpload(file_req) => {
                        // 模拟全部同意
                        let agreed_ids = file_req
                            .files
                            .into_keys()
                            .map(|file_id| file_id)
                            .collect::<HashSet<String>>();
                        let _ = out_tx.send(OutMessage::FileAgreedUpload(agreed_ids)).await;
                    }
                }
            }
        }
    });
    server.start().await.unwrap();
}
