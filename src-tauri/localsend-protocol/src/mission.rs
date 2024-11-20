use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::model::{DeviceMessage, FileInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub sender: DeviceMessage,
    pub id_token_map: HashMap<String, String>,
    pub token_id_map: HashMap<String, String>,
    pub info_map: HashMap<String, FileInfo>,
}

impl Mission {
    pub fn new(info_map: HashMap<String, FileInfo>, sender: DeviceMessage) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let mut id_token_map = HashMap::new();
        let mut token_id_map = HashMap::new();
        info_map.iter().for_each(|(id, _value)| {
            let token = uuid::Uuid::new_v4().to_string();
            id_token_map.insert(id.clone(), token.clone());
            token_id_map.insert(token, id.clone());
        });
        Self {
            id,
            sender,
            id_token_map,
            token_id_map,
            info_map,
        }
    }
}