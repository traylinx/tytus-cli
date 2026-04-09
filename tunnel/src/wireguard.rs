//! WireGuard tunnel implementation using boringtun + tun crate.
//!
//! Architecture:
//! - TUN device: created via `tun` crate (platform-specific: utun on macOS, /dev/net/tun on Linux)
//! - UDP socket: sends/receives encrypted WireGuard packets to/from the server
//! - boringtun Tunn: handles encryption/decryption (Noise protocol)
//! - Packet loop: tokio::select! multiplexes TUN reads, UDP reads, and timer ticks

use crate::{TunnelConfig, TunnelHandle, TunnelState};
use atomek_core::AtomekError;
use boringtun::noise::{Tunn, TunnResult};
use std::net::{SocketAddr, UdpSocket as StdUdpSocket};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;
#[allow(unused_imports)]
use tun::Device as _; // needed for tun_name() method on AsyncDevice

const MAX_PACKET: usize = 65535;
const TIMER_TICK_MS: u64 = 1000;

enum LoopAction {
    WriteToTun(Vec<u8>),
    SendUdp(Vec<u8>),
}

/// Create TUN device, UDP socket, boringtun Tunn, and start packet forwarding loop.
pub async fn create_tunnel(config: TunnelConfig) -> Result<TunnelHandle, AtomekError> {
    // 1. Parse keys from base64
    let private_key = parse_static_secret(&config.private_key, "PrivateKey")?;
    let peer_public_key = parse_public_key(&config.peer_public_key, "PublicKey")?;
    let preshared_key: Option<[u8; 32]> = config.preshared_key.as_ref()
        .map(|k| parse_key_bytes(k, "PresharedKey"))
        .transpose()?;

    // 2. Parse network config
    let local_ip = config.address.split('/').next()
        .ok_or_else(|| AtomekError::Other("Invalid address format".into()))?
        .to_string();
    let cidr: u8 = config.address.split('/').nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(24);

    let endpoint: SocketAddr = config.endpoint.parse()
        .map_err(|e| AtomekError::Other(format!("Invalid endpoint '{}': {}", config.endpoint, e)))?;

    tracing::info!(
        local_ip = %local_ip,
        endpoint = %endpoint,
        "Creating WireGuard tunnel"
    );

    // 3. Create TUN device
    let mut tun_config = tun::Configuration::default();
    tun_config.address(&local_ip)
        .netmask(cidr_to_netmask(cidr))
        .destination(&local_ip) // point-to-point
        .mtu(1420) // WireGuard standard MTU
        .up();

    #[cfg(target_os = "linux")]
    tun_config.platform_config(|p| {
        p.ensure_root_privileges(true);
    });

    let tun_device = match tun::create_as_async(&tun_config) {
        Ok(dev) => dev,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("permission") || msg.contains("Operation not permitted") {
                return Err(AtomekError::PrivilegesRequired);
            } else {
                return Err(AtomekError::Tunnel(format!("Failed to create TUN device: {}", msg)));
            }
        }
    };

    let interface_name = tun_device.as_ref().tun_name()
        .map_err(|e| AtomekError::Tunnel(format!("Failed to get TUN name: {}", e)))?
        .to_string();

    tracing::info!(interface = %interface_name, "TUN device created");

    // 3b. Add route for allowed IPs through the TUN interface
    // allowed_ips is a single string like "10.18.1.0/24" — may contain commas for multiple
    let allowed_ip_list: Vec<&str> = config.allowed_ips.split(',').map(|s| s.trim()).collect();
    for allowed_ip in &allowed_ip_list {
        let network = allowed_ip.split('/').next().unwrap_or(allowed_ip);
        let cidr_bits = allowed_ip.split('/').nth(1).unwrap_or("24");
        tracing::info!(route = %allowed_ip, interface = %interface_name, "Adding route");

        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("/sbin/route")
                .args(["-n", "add", "-net", &format!("{}/{}", network, cidr_bits), "-interface", &interface_name])
                .output();
            match output {
                Ok(o) if o.status.success() => tracing::info!("Route added: {}", allowed_ip),
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    tracing::warn!("Route add exited with {}: {}", o.status, stderr.trim());
                }
                Err(e) => tracing::warn!("Failed to add route: {}", e),
            }
        }

        #[cfg(target_os = "linux")]
        {
            let output = std::process::Command::new("/sbin/ip")
                .args(["route", "add", allowed_ip, "dev", &interface_name])
                .output();
            match output {
                Ok(o) if o.status.success() => tracing::info!("Route added: {}", allowed_ip),
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    tracing::warn!("Route add exited with {}: {}", o.status, stderr.trim());
                }
                Err(e) => tracing::warn!("Failed to add route: {}", e),
            }
        }
    }

    // 4. Create UDP socket for WireGuard traffic
    let std_socket = StdUdpSocket::bind("0.0.0.0:0")
        .map_err(|e| AtomekError::Tunnel(format!("Failed to bind UDP socket: {}", e)))?;
    std_socket.set_nonblocking(true)
        .map_err(|e| AtomekError::Tunnel(format!("Failed to set non-blocking: {}", e)))?;

    let udp_socket = UdpSocket::from_std(std_socket)
        .map_err(|e| AtomekError::Tunnel(format!("Failed to create async UDP socket: {}", e)))?;

    tracing::info!(
        local_addr = %udp_socket.local_addr().unwrap(),
        peer = %endpoint,
        "UDP socket ready"
    );

    // 5. Create boringtun Tunn
    let keepalive = config.persistent_keepalive;

    let tunn = Tunn::new(
        private_key,
        peer_public_key,
        preshared_key,
        keepalive,
        0, // index — single peer, always 0
        None, // rate limiter
    );

    let tunn = Arc::new(std::sync::Mutex::new(tunn));

    // 6. Initiate handshake
    let handshake_data = {
        let mut t = tunn.lock().unwrap_or_else(|e| e.into_inner());
        let mut buf = vec![0u8; MAX_PACKET];
        match t.format_handshake_initiation(&mut buf, false) {
            TunnResult::WriteToNetwork(data) => Some(data.to_vec()),
            _ => {
                tracing::warn!("Unexpected result from format_handshake_initiation");
                None
            }
        }
    };
    if let Some(data) = handshake_data {
        udp_socket.send_to(&data, endpoint).await
            .map_err(|e| AtomekError::Tunnel(format!("Failed to send handshake: {}", e)))?;
        tracing::info!("Handshake initiation sent to {}", endpoint);
    }

    // 7. Start packet forwarding loop
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();
    let iface_name = interface_name.clone();
    let local_ip_clone = local_ip.clone();

    let task = tokio::spawn(async move {
        if let Err(e) = packet_loop(tun_device, udp_socket, tunn, endpoint, cancel_clone).await {
            tracing::error!("Tunnel packet loop failed: {}", e);
        }
        tracing::info!("Tunnel packet loop ended");
    });

    Ok(TunnelHandle {
        cancel,
        task,
        state: TunnelState::Up {
            interface_name: iface_name,
            local_ip: local_ip_clone,
        },
        interface_name,
    })
}

/// Main packet forwarding loop.
/// Multiplexes: TUN reads ↔ UDP socket ↔ timer ticks.
async fn packet_loop(
    tun_device: tun::AsyncDevice,
    udp_socket: UdpSocket,
    tunn: Arc<std::sync::Mutex<Tunn>>,
    endpoint: SocketAddr,
    cancel: CancellationToken,
) -> Result<(), AtomekError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let (mut tun_reader, mut tun_writer) = tokio::io::split(tun_device);
    let mut tun_buf = vec![0u8; MAX_PACKET];
    let mut udp_buf = vec![0u8; MAX_PACKET];
    let mut out_buf = vec![0u8; MAX_PACKET];

    let mut timer_interval = tokio::time::interval(
        std::time::Duration::from_millis(TIMER_TICK_MS)
    );

    let mut handshake_complete = false;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Tunnel cancelled");
                break;
            }

            // TUN → encrypt → UDP (outgoing traffic)
            result = tun_reader.read(&mut tun_buf) => {
                match result {
                    Ok(n) if n > 0 => {
                        let send_data = {
                            let mut t = tunn.lock().unwrap_or_else(|e| e.into_inner());
                            match t.encapsulate(&tun_buf[..n], &mut out_buf) {
                                TunnResult::WriteToNetwork(data) => Some(data.to_vec()),
                                TunnResult::Err(e) => {
                                    tracing::debug!("Encapsulate error: {:?}", e);
                                    None
                                }
                                _ => None,
                            }
                        }; // lock released here
                        if let Some(data) = send_data {
                            let _ = udp_socket.send_to(&data, endpoint).await;
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("TUN read error: {}", e);
                    }
                }
            }

            // UDP → decrypt → TUN (incoming traffic)
            result = udp_socket.recv_from(&mut udp_buf) => {
                match result {
                    Ok((n, _src)) => {
                        // Process decapsulation chain — collect actions then execute
                        let actions = {
                            let mut t = tunn.lock().unwrap_or_else(|e| e.into_inner());
                            let mut actions = Vec::new();
                            let mut res = t.decapsulate(None, &udp_buf[..n], &mut out_buf);
                            loop {
                                match res {
                                    TunnResult::WriteToTunnelV4(data, _) | TunnResult::WriteToTunnelV6(data, _) => {
                                        actions.push(LoopAction::WriteToTun(data.to_vec()));
                                        break;
                                    }
                                    TunnResult::WriteToNetwork(data) => {
                                        actions.push(LoopAction::SendUdp(data.to_vec()));
                                        res = t.decapsulate(None, &[], &mut out_buf);
                                    }
                                    TunnResult::Done => break,
                                    TunnResult::Err(e) => {
                                        tracing::debug!("Decapsulate error: {:?}", e);
                                        break;
                                    }
                                }
                            }
                            actions
                        }; // lock released

                        for action in actions {
                            match action {
                                LoopAction::WriteToTun(data) => {
                                    if !handshake_complete {
                                        handshake_complete = true;
                                        tracing::info!("WireGuard handshake complete — tunnel active");
                                    }
                                    if let Err(e) = tun_writer.write_all(&data).await {
                                        tracing::debug!("TUN write error: {}", e);
                                    }
                                }
                                LoopAction::SendUdp(data) => {
                                    if let Err(e) = udp_socket.send_to(&data, endpoint).await {
                                        tracing::debug!("UDP send error: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("UDP recv error: {}", e);
                    }
                }
            }

            // Timer tick — keepalive, rekey, handshake retransmit
            _ = timer_interval.tick() => {
                let packets = {
                    let mut t = tunn.lock().unwrap_or_else(|e| e.into_inner());
                    let mut packets = Vec::new();
                    let mut result = t.update_timers(&mut out_buf);
                    while let TunnResult::WriteToNetwork(data) = result {
                        packets.push(data.to_vec());
                        result = t.decapsulate(None, &[], &mut out_buf);
                    }
                    packets
                }; // lock released
                for pkt in packets {
                    let _ = udp_socket.send_to(&pkt, endpoint).await;
                }
            }
        }
    }

    Ok(())
}

// ── Helpers ──

fn parse_key_bytes(b64: &str, name: &str) -> Result<[u8; 32], AtomekError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(b64.trim())
        .map_err(|e| AtomekError::Other(format!("Invalid {} base64: {}", name, e)))?;

    if bytes.len() != 32 {
        return Err(AtomekError::Other(format!("{} must be 32 bytes, got {}", name, bytes.len())));
    }

    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(arr)
}

fn parse_static_secret(b64: &str, name: &str) -> Result<x25519_dalek::StaticSecret, AtomekError> {
    let arr = parse_key_bytes(b64, name)?;
    Ok(x25519_dalek::StaticSecret::from(arr))
}

fn parse_public_key(b64: &str, name: &str) -> Result<x25519_dalek::PublicKey, AtomekError> {
    let arr = parse_key_bytes(b64, name)?;
    Ok(x25519_dalek::PublicKey::from(arr))
}

fn cidr_to_netmask(cidr: u8) -> String {
    let mask: u32 = if cidr >= 32 { 0xFFFFFFFF } else { !((1u32 << (32 - cidr)) - 1) };
    format!("{}.{}.{}.{}",
        (mask >> 24) & 0xFF,
        (mask >> 16) & 0xFF,
        (mask >> 8) & 0xFF,
        mask & 0xFF,
    )
}

