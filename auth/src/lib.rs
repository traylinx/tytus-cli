pub mod device_auth;
pub mod keychain;
pub mod login;
pub mod sentinel;

// Device auth flow (primary — no password)
pub use device_auth::{
    create_device_session, poll_for_authorization, refresh_access_token, validate_token,
    DeviceAuthResult, DeviceAuthSession, DeviceAuthUser, TokenValidation,
};

// Wannolot pass + credentials
pub use sentinel::{fetch_wannolot_pass, plan_from_pod_status, PlanStatus, SentinelCredentials};

// Legacy login (kept for testing/fallback)
pub use login::{login, refresh_token, LoginResult, UserInfo};

// Keychain
pub use keychain::KeychainStore;
