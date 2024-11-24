use std::{collections::HashMap, net::SocketAddr, path::PathBuf};

use localsend_protocol::{
    mission::Mission,
    model::{DeviceMessage, DeviceType},
    server::ServerSetting,
};
use tokio::sync::RwLock;

pub struct AppState {
    pub setting: RwLock<ServerSetting>,
    pub devices: RwLock<HashMap<String, (SocketAddr, DeviceMessage)>>,
    pub misssions: RwLock<HashMap<String, Mission>>,
}

impl AppState {
    pub fn new(store_path: PathBuf) -> Self {
        let hostname = tauri_plugin_os::hostname();
        let device_type = match tauri_plugin_os::platform() {
            "windows" | "macos" | "linux" => DeviceType::Desktop,
            "ios" | "android" => DeviceType::Mobile,
            _ => DeviceType::Headless,
        };
        let settings = ServerSetting {
            alias: hostname,
            device_type: Some(device_type),
            store_path,
            ..Default::default()
        };
        AppState {
            setting: RwLock::new(settings),
            devices: RwLock::new(HashMap::new()),
            misssions: RwLock::new(HashMap::new()),
        }
    }
}
