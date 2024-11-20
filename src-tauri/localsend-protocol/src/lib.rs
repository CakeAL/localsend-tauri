pub mod api;
pub mod mission;
pub mod model;
pub mod multicast;
pub mod server;

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use crate::{
        model::DeviceType,
        server::{Server, ServerSetting},
    };

    #[tokio::test]
    async fn test_server() {
        let (_tx, rx) = mpsc::channel(8);
        let setting = ServerSetting {
            alias: "test_device".to_string(),
            device_model: Some("test_model".to_string()),
            device_type: Some(DeviceType::Headless),
            protocol: Some(crate::model::Protocol::Http),
            download: false,
            ..Default::default()
        };
        let (server, _rx) = Server::new(setting, rx);
        server.start().await.unwrap();
    }
}
