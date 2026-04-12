pub mod wireguard;
pub mod monitor;

use atomek_core::AtomekError;

/// Configuration parsed from WireGuard .conf file
#[derive(Debug, Clone)]
pub struct TunnelConfig {
    pub private_key: String,
    pub address: String,       // e.g. "10.X.Y.2/24" — peer address inside the tunnel
    pub dns: Option<String>,
    pub peer_public_key: String,
    pub preshared_key: Option<String>,
    pub endpoint: String,      // e.g. "<droplet-public-ip>:51800+podnum"
    pub allowed_ips: String,   // e.g. "10.X.Y.0/24, 10.42.42.1/32" — destinations to route through this tunnel
    pub persistent_keepalive: Option<u16>,
}

/// Runtime state of the tunnel
#[derive(Debug, Clone, PartialEq)]
pub enum TunnelState {
    Down,
    Connecting,
    Up {
        interface_name: String,
        local_ip: String,
    },
    Failed(String),
}

/// Handle to a running tunnel. Call `.shutdown()` to gracefully stop it.
pub struct TunnelHandle {
    pub(crate) cancel: tokio_util::sync::CancellationToken,
    pub(crate) task: tokio::task::JoinHandle<()>,
    pub state: TunnelState,
    pub interface_name: String,
}

impl TunnelHandle {
    /// Gracefully shut down the tunnel
    pub async fn shutdown(self) {
        tracing::info!("Shutting down WireGuard tunnel");
        self.cancel.cancel();
        let _ = self.task.await;
        tracing::info!("Tunnel shut down");
    }

    /// Borrow the cancel token so the caller can trigger shutdown without
    /// consuming the handle. Used by FIX-4 in tytus-cli where cmd_tunnel_up
    /// needs to race ctrl_c vs. the packet-loop task finishing.
    pub fn cancel_token(&self) -> tokio_util::sync::CancellationToken {
        self.cancel.clone()
    }

    /// Take ownership of the spawned packet-loop task. After calling this,
    /// `shutdown()` will still work (it's a no-op on the already-taken task
    /// but will still fire the cancel token). Intended for callers that
    /// want to `select!` on the task alongside other futures.
    pub fn take_task(&mut self) -> tokio::task::JoinHandle<()> {
        std::mem::replace(&mut self.task, tokio::spawn(async {}))
    }
}

/// Create and activate a WireGuard tunnel.
/// Returns a handle that can be used to shut it down.
pub async fn connect(config: TunnelConfig) -> Result<TunnelHandle, AtomekError> {
    wireguard::create_tunnel(config).await
}
