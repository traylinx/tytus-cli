//! Channel registry — which chat/message channels `tytus channels add`
//! knows how to configure.
//!
//! Each entry maps a user-facing channel name (e.g. `telegram`) to the
//! set of environment variables the agent's channel extension expects
//! at startup. The env-var names come from each extension's
//! `openclaw.plugin.json` → `channelEnvVars` field — they're the
//! contract between Tytus and the extension.
//!
//! Adding a new channel here is a 3-line change; the rest of the
//! plumbing (keychain storage, `.tytus/channels.json` on the pod,
//! DAM redeploy) is channel-agnostic.

/// One credential the user must provide when enabling a channel.
pub struct Credential {
    /// Env var name. Must match what the OpenClaw/Hermes plugin reads.
    pub env_var: &'static str,
    /// Short description shown in `tytus channels add` help text.
    pub label: &'static str,
    /// CLI flag name (without `--` prefix). The primary credential is
    /// always called `--token`; additional ones get descriptive names.
    pub cli_flag: &'static str,
}

/// One configurable channel — maps a short name to the credentials
/// needed and a blurb shown to the user.
pub struct ChannelSpec {
    /// The short name users type: `telegram`, `discord`, `slack`, …
    pub name: &'static str,
    /// Human-friendly label for menu + help output.
    pub label: &'static str,
    /// One line of "what this is + how to get credentials."
    pub blurb: &'static str,
    /// Credentials the user must supply. First one is always exposed
    /// as `--token`; the rest use `--<cli_flag>`.
    pub credentials: &'static [Credential],
    /// Which agent types support this channel. Today only `nemoclaw`
    /// (OpenClaw) — Hermes has its own channel integrations that
    /// don't follow OpenClaw's env-var convention.
    pub agent_types: &'static [&'static str],
    /// Short text explaining how the channel delivers inbound
    /// messages. Useful for users asking "why does X work without a
    /// public URL but Y doesn't?"
    pub inbound_model: &'static str,
}

/// The registry itself. Add an entry here to ship a new channel.
///
/// MVP scope: channels that work **without a public pod webhook
/// endpoint**. Webhook-requiring channels (WhatsApp Business,
/// Slack Events API, Meta stuff) are deferred to the sprint that
/// ships the per-pod public subdomain — see
/// `services/tytus-cli/dev/design/2026-04-20-unblock-openclaw-channels.md`.
pub const REGISTRY: &[ChannelSpec] = &[
    ChannelSpec {
        name: "telegram",
        label: "Telegram",
        blurb: "Register a bot with @BotFather on Telegram and copy the bot token.",
        credentials: &[
            Credential {
                env_var: "TELEGRAM_BOT_TOKEN",
                label: "Bot token (from @BotFather)",
                cli_flag: "token",
            },
        ],
        agent_types: &["nemoclaw"],
        inbound_model: "Long-polling — no public endpoint needed. Works while the pod is up, even if your laptop is closed.",
    },
    ChannelSpec {
        name: "discord",
        label: "Discord",
        blurb: "Create an application + bot at https://discord.com/developers/applications and copy the bot token.",
        credentials: &[
            Credential {
                env_var: "DISCORD_BOT_TOKEN",
                label: "Bot token",
                cli_flag: "token",
            },
        ],
        agent_types: &["nemoclaw"],
        inbound_model: "Gateway (persistent WebSocket pod → Discord). No public endpoint needed.",
    },
    ChannelSpec {
        name: "slack",
        label: "Slack (Socket Mode)",
        blurb: "Create a Slack app with Socket Mode enabled; grab the bot token + app-level token.",
        credentials: &[
            Credential {
                env_var: "SLACK_BOT_TOKEN",
                label: "Bot User OAuth token (xoxb-...)",
                cli_flag: "token",
            },
            Credential {
                env_var: "SLACK_APP_TOKEN",
                label: "App-level token (xapp-...) — required for Socket Mode",
                cli_flag: "app-token",
            },
            Credential {
                env_var: "SLACK_USER_TOKEN",
                label: "User OAuth token (xoxp-...) — optional, for user-scoped actions",
                cli_flag: "user-token",
            },
        ],
        agent_types: &["nemoclaw"],
        inbound_model: "Socket Mode — no public endpoint needed. Events flow over a persistent WebSocket.",
    },
    ChannelSpec {
        name: "line",
        label: "LINE",
        blurb: "Create a LINE Messaging API channel and copy the channel access token + channel secret.",
        credentials: &[
            Credential {
                env_var: "LINE_CHANNEL_ACCESS_TOKEN",
                label: "Channel access token",
                cli_flag: "token",
            },
            Credential {
                env_var: "LINE_CHANNEL_SECRET",
                label: "Channel secret",
                cli_flag: "channel-secret",
            },
        ],
        agent_types: &["nemoclaw"],
        inbound_model: "Webhook — a public pod endpoint is required (not yet available). Outbound works today.",
    },
];

/// Look up a channel by user-typed name. Case-insensitive.
pub fn find(name: &str) -> Option<&'static ChannelSpec> {
    let lower = name.to_lowercase();
    REGISTRY.iter().find(|c| c.name == lower)
}

/// Return the list of channels that work for a given agent type.
/// Used by `tytus channels add --help` and the tray wizard.
#[allow(dead_code)]
pub fn for_agent(agent_type: &str) -> Vec<&'static ChannelSpec> {
    REGISTRY
        .iter()
        .filter(|c| c.agent_types.contains(&agent_type))
        .collect()
}
