use core::str;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Mobile,
    Desktop,
    Web,
    Headless,
    Server,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    Http,
    Https,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub alias: String,
    pub version: String, // protocol version (major.minor)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_type: Option<DeviceType>,
    pub fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")] // HTTP Legancy Mode Resp: None
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")] // HTTP Legancy Mode Resp: None
    pub protocol: Option<Protocol>,
    pub download: bool, // if download API (section 5.2, 5.3) is active (optional, default: false)
    #[serde(skip_serializing_if = "Option::is_none")] // Multicast: Some(true/false)
    pub announce: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRequest {
    pub info: Message,
    pub files: HashMap<String, FileInfo>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub id: String,
    pub file_name: String,
    pub size: u64, // bytes
    pub file_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResponse {
    pub session_id: String,
    pub files: HashMap<String, String>,
}