use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirewallPolicy {
    pub loopback_bind: IpAddr,
    pub requires_private_tunnel_permission: bool,
    pub repair_action: &'static str,
}

pub fn default_policy() -> FirewallPolicy {
    FirewallPolicy {
        loopback_bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
        requires_private_tunnel_permission: private_tunnel_permission_required(),
        repair_action: "repair_private_connection",
    }
}

pub fn private_tunnel_permission_required() -> bool {
    // Consumer Windows path owns Wintun/WireGuard service setup under one UAC
    // prompt. macOS/Linux may still need helper/polkit in Phase 4, but Phase 2
    // exposes the decision surface without implementing helper elevation here.
    cfg!(target_os = "windows")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_policy_never_binds_publicly() {
        let p = default_policy();
        assert_eq!(p.loopback_bind, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(p.repair_action, "repair_private_connection");
    }
}
