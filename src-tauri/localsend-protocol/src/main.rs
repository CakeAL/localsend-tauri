use tokio::sync::mpsc;

use localsend_protocol::{
    model::DeviceType,
    server::{Server, ServerSetting},
};

// for test
#[tokio::main]
async fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();
    let (_tx, rx) = mpsc::channel(8);
    let setting = ServerSetting {
        alias: "test_device".to_string(),
        device_model: Some("test_model".to_string()),
        device_type: Some(DeviceType::Headless),
        protocol: Some(localsend_protocol::model::Protocol::Http),
        download: false,
        ..Default::default()
    };
    let (server, _rx) = Server::new(setting, rx);
    server.start().await.unwrap();
}
