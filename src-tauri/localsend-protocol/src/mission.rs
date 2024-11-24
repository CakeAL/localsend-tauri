use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{DeviceMessage, FileInfo};

// 下载任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub sender_device: DeviceMessage,
    pub id_token_map: HashMap<String, String>,
    pub info_map: HashMap<String, FileInfo>,
}

impl Mission {
    pub fn new(info_map: HashMap<String, FileInfo>, sender_device: DeviceMessage) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let mut id_token_map = HashMap::new();
        info_map.iter().for_each(|(id, _value)| {
            let token = uuid::Uuid::new_v4().to_string();
            id_token_map.insert(id.clone(), token.clone());
        });
        Self {
            id,
            sender_device,
            id_token_map,
            info_map,
        }
    }
}
