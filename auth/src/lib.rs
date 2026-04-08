pub mod device_auth;
pub mod login;
pub mod sentinel;
pub mod keychain;

// Device auth flow (primary — no password)
pub use device_auth::{
    create_device_session, poll_for_authorization, refresh_access_token,
    DeviceAuthSession, DeviceAuthResult, DeviceAuthUser,
};

// Wannolot pass + credentials
pub use sentinel::{fetch_wannolot_pass, plan_from_pod_status, SentinelCredentials, PlanStatus};

// Legacy login (kept for testing/fallback)
pub use login::{login, refresh_token, LoginResult, UserInfo};

// Keychain
pub use keychain::KeychainStore;
