use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use tokio::net::UdpSocket;

use crate::model::DeviceMessage;

pub async fn multicast_message(
    recv_addr: &SocketAddrV4,
    message: &DeviceMessage,
) -> io::Result<()> {
    let ipv4s = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|iface| match iface.addr {
            if_addrs::IfAddr::V4(v4) if !v4.is_loopback() => Some(v4.ip.into()),
            _ => None,
        })
        .collect::<Vec<Ipv4Addr>>();
    // dbg!(&ipv4s);
    for ipv4 in ipv4s {
        let local_addr = SocketAddrV4::new(ipv4, 0);
        let socket = UdpSocket::bind(&local_addr).await?;
        let message = serde_json::json!(message).to_string();
        // log::info!("Send multicast message on {:?}", socket);
        // 多发几次
        for _ in 0..5 {
            socket.send_to(message.as_bytes(), &recv_addr).await?;
        }
    }
    Ok(())
}

pub async fn multicast_listener(addr: &SocketAddrV4) -> io::Result<(DeviceMessage, SocketAddr)> {
    let local_addr = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, addr.port());
    let socket = UdpSocket::bind(&local_addr).await?;
    let ipv4s = if_addrs::get_if_addrs()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|iface| match iface.addr {
            if_addrs::IfAddr::V4(v4) if !v4.is_loopback() => Some(v4.ip.into()),
            _ => None,
        })
        .collect::<Vec<Ipv4Addr>>();
    for iface in ipv4s.iter() {
        socket.join_multicast_v4(addr.ip().to_owned(), *iface)?;
    }
    let mut buf = vec![0u8; 1024];
    log::info!("start multicast listening on {:?}", socket);
    let (len, sender_addr) = socket.recv_from(&mut buf).await?;
    let message = serde_json::from_slice::<DeviceMessage>(&buf[..len]).unwrap_or_default();
    log::info!("accept message from multicast: {:?}", message);
    Ok((message, sender_addr))
}

#[cfg(test)]
mod tests {
    use crate::model::*;

    use super::*;

    #[tokio::test]
    async fn test_send_message() {
        let recv: SocketAddrV4 = "224.0.0.167:53317".parse().unwrap();
        let message = DeviceMessage {
            alias: "test-client".to_string(),
            version: "2.1".to_string(),
            device_model: Some("Test Model".to_string()),
            device_type: Some(DeviceType::Desktop),
            fingerprint: "test-client-fingerprint".to_string(),
            port: Some(53317),
            protocol: Some(Protocol::Http),
            download: true,
            announce: Some(true),
        };
        multicast_message(&recv, &message).await.unwrap();
    }

    #[tokio::test]
    async fn test_receive_message() {
        // let _ = env_logger::builder()
        //     .filter_level(log::LevelFilter::Info)
        //     .is_test(true)
        //     .try_init();
        let addr: SocketAddrV4 = "224.0.0.167:53317".parse().unwrap();
        for _i in 0..5 {
            let (message, sender_addr) = multicast_listener(&addr).await.unwrap();
            dbg!(message, sender_addr);
        }
    }
}
