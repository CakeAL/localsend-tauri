use tokio::sync::mpsc;

use localsend_protocol::{
    model::DeviceType,
    request::send_register,
    server::{Server, ServerMessage, ServerSetting},
};

// for test
#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();
    let (_tx, out_rx) = mpsc::channel(8);
    let setting = ServerSetting {
        alias: "test_device".to_string(),
        device_model: Some("test_model".to_string()),
        device_type: Some(DeviceType::Headless),
        protocol: Some(localsend_protocol::model::Protocol::Http),
        download: false,
        ..Default::default()
    };
    let (server, mut server_rx) = Server::new(setting.clone(), out_rx);
    let _ = tokio::spawn(async move {
        loop {
            match server_rx.recv().await {
                Some(message) => {
                    match message {
                        // 服务器监听到连接请求
                        ServerMessage::DeviceConnect(addr, _device) => {
                            if let Err(e) = send_register(&setting, &addr).await {
                                log::error!("send register error: {e:?}");
                            }
                        }
                    }
                }
                None => {}
            }
        }
    });
    server.start().await.unwrap();
}
