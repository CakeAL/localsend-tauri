use std::net::IpAddr;

use localsend_protocol::server::OutMessage;

use crate::model::AppState;

#[tauri::command(async)]
pub async fn get_device_info(app_state: tauri::State<'_, AppState>) -> Result<String, String> {
    let device = app_state.setting.read().await.to_device_message(None);
    let ipv4s = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|iface| match iface.addr {
            if_addrs::IfAddr::V4(v4) if !v4.is_loopback() => Some(v4.ip.into()),
            _ => None,
        })
        .collect::<Vec<IpAddr>>();

    Ok(serde_json::json!((device, ipv4s)).to_string())
}

#[tauri::command(async)]
pub async fn refresh(app_state: tauri::State<'_, AppState>) -> Result<(), String> {
    match app_state.sender.read().await.as_ref() {
        Some(sender) => {
            let _ = sender.send(OutMessage::Refresh).await;
        }
        None => {
            log::error!("OutMessage Sender is None?");
        }
    }
    Ok(())
}
