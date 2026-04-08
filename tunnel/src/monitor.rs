//! Tunnel health monitoring.
//! Checks if the WireGuard handshake is still alive by pinging the gateway.

use std::net::IpAddr;
use std::time::Duration;

/// Check if the tunnel is healthy by attempting to reach the gateway IP.
/// Returns true if we can open a TCP connection to port 18080 (switchAILocal).
pub async fn check_tunnel_health(gateway_ip: &str) -> bool {
    let addr: IpAddr = match gateway_ip.parse() {
        Ok(a) => a,
        Err(_) => return false,
    };

    // Try connecting to switchAILocal on port 18080
    let socket_addr = std::net::SocketAddr::new(addr, 18080);
    match tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::TcpStream::connect(socket_addr),
    ).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

/// Extract the gateway IP from a subnet string.
/// "10.17.8.0/24" → "10.17.8.1"
pub fn gateway_from_subnet(subnet: &str) -> Option<String> {
    let base = subnet.split('/').next()?;
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() == 4 {
        Some(format!("{}.{}.{}.1", parts[0], parts[1], parts[2]))
    } else {
        None
    }
}
