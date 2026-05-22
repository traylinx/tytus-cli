use crate::state::{AccountProfile, CliState};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AccountListEntry {
    pub email: String,
    pub active: bool,
    pub has_token: bool,
    pub pod_count: usize,
    pub last_used_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AccountListOutput {
    pub active_email: Option<String>,
    pub accounts: Vec<AccountListEntry>,
}

pub fn canonical_email(email: &str) -> String {
    email.trim().to_ascii_lowercase()
}

pub fn profile_from_state(state: &CliState) -> Option<AccountProfile> {
    let email = state.email.as_ref()?.trim();
    if email.is_empty() {
        return None;
    }
    Some(AccountProfile {
        email: email.to_string(),
        access_token: state.access_token.clone(),
        expires_at_ms: state.expires_at_ms,
        secret_key: state.secret_key.clone(),
        agent_user_id: state.agent_user_id.clone(),
        organization_id: state.organization_id.clone(),
        tier: state.tier.clone(),
        pods: state.pods.clone(),
        device_session_id: state.device_session_id,
        last_active_at: Some(chrono::Utc::now()),
    })
}

pub fn upsert_profile(state: &mut CliState, profile: AccountProfile) {
    if let Some(existing) = state.accounts.iter_mut().find(|p| p.email == profile.email) {
        *existing = profile;
    } else {
        state.accounts.push(profile);
    }
}

pub fn remove_profile(state: &mut CliState, email: &str) -> Option<AccountProfile> {
    let pos = state.accounts.iter().position(|p| p.email == email)?;
    Some(state.accounts.remove(pos))
}

pub fn list_accounts(state: &CliState) -> AccountListOutput {
    list_accounts_with(
        state,
        atomek_auth::KeychainStore::find_stored_email(),
        atomek_auth::KeychainStore::list_indexed_accounts(),
        |email| atomek_auth::KeychainStore::try_get_refresh_token(email).is_some(),
    )
}

pub fn list_accounts_with<F>(
    state: &CliState,
    last_email: Option<String>,
    indexed_emails: Vec<String>,
    has_token_fn: F,
) -> AccountListOutput
where
    F: Fn(&str) -> bool,
{
    let mut emails: Vec<String> = state.accounts.iter().map(|a| a.email.clone()).collect();
    if let Some(email) = last_email {
        if !emails.iter().any(|e| e == &email) {
            emails.push(email);
        }
    }
    for email in indexed_emails {
        if !emails.iter().any(|e| e == &email) {
            emails.push(email);
        }
    }
    emails.sort();
    emails.dedup();

    let accounts = emails
        .into_iter()
        .map(|email| {
            let profile = state.accounts.iter().find(|a| a.email == email);
            let active = state.active_email.as_deref() == Some(email.as_str());
            let has_token = has_token_fn(&email);
            let pod_count = profile.map(|p| p.pods.len()).unwrap_or(0);
            let last_used_at = profile.and_then(|p| p.last_active_at.map(|t| t.to_rfc3339()));
            let status = if active {
                "active"
            } else if profile.is_none() {
                "orphan"
            } else if !has_token {
                "no_token"
            } else {
                "stored"
            }
            .to_string();
            AccountListEntry {
                email,
                active,
                has_token,
                pod_count,
                last_used_at,
                status,
            }
        })
        .collect();

    AccountListOutput {
        active_email: state.active_email.clone(),
        accounts,
    }
}

pub fn render_human_list(output: &AccountListOutput) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{:<36} {:<10} {:<9} {}\n",
        "EMAIL", "STATUS", "PODS", "LAST USED"
    ));
    for account in &output.accounts {
        let marker = if account.active { "* " } else { "  " };
        let pods = if account.status == "orphan" {
            "-".to_string()
        } else {
            account.pod_count.to_string()
        };
        let status = match account.status.as_str() {
            "orphan" => "(orphan)",
            "no_token" => "(no token)",
            other => other,
        };
        out.push_str(&format!(
            "{}{:<34} {:<10} {:<9} {}\n",
            marker,
            account.email,
            status,
            pods,
            account.last_used_at.as_deref().unwrap_or("-")
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_email_lowercases_and_trims() {
        assert_eq!(canonical_email(" User@Example.COM "), "user@example.com");
    }

    fn sample_state() -> CliState {
        CliState {
            schema_version: 2,
            active_email: Some("a@example.com".into()),
            accounts: vec![
                AccountProfile {
                    email: "a@example.com".into(),
                    pods: vec![Default::default(), Default::default()],
                    last_active_at: Some(
                        chrono::DateTime::parse_from_rfc3339("2026-04-29T18:30:00Z")
                            .unwrap()
                            .with_timezone(&chrono::Utc),
                    ),
                    ..Default::default()
                },
                AccountProfile {
                    email: "b@example.com".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn list_accounts_marks_active_stored_orphan_and_no_token() {
        let out = list_accounts_with(
            &sample_state(),
            Some("orphan@example.com".into()),
            vec!["indexed@example.com".into()],
            |email| {
                matches!(
                    email,
                    "a@example.com" | "indexed@example.com" | "orphan@example.com"
                )
            },
        );
        assert_eq!(out.active_email.as_deref(), Some("a@example.com"));
        let active = out
            .accounts
            .iter()
            .find(|a| a.email == "a@example.com")
            .unwrap();
        assert!(active.active);
        assert_eq!(active.status, "active");
        assert_eq!(active.pod_count, 2);
        let no_token = out
            .accounts
            .iter()
            .find(|a| a.email == "b@example.com")
            .unwrap();
        assert_eq!(no_token.status, "no_token");
        let orphan = out
            .accounts
            .iter()
            .find(|a| a.email == "orphan@example.com")
            .unwrap();
        assert_eq!(orphan.status, "orphan");
    }

    #[test]
    fn human_list_uses_requested_labels() {
        let out = list_accounts_with(
            &sample_state(),
            Some("orphan@example.com".into()),
            vec![],
            |email| email == "a@example.com",
        );
        let human = render_human_list(&out);
        assert!(human.contains("* a@example.com"));
        assert!(human.contains("(no token)"));
        assert!(human.contains("(orphan)"));
    }
}
