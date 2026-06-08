use crate::state::{CliState, PodEntry};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPodTarget {
    pub pod_id: String,
    pub route_id: Option<String>,
    pub display_name: Option<String>,
    pub agent_type: Option<String>,
}

impl ResolvedPodTarget {
    fn from_pod(pod: &PodEntry) -> Self {
        Self {
            pod_id: pod.pod_id.clone(),
            route_id: pod.route_id.clone(),
            display_name: pod.display_name.clone(),
            agent_type: pod.agent_type.clone(),
        }
    }

    pub fn agent_label(&self) -> &'static str {
        match self.agent_type.as_deref() {
            Some("nemoclaw") => "OpenClaw",
            Some("hermes") => "Hermes",
            Some("none") => "AIL",
            _ => "unknown",
        }
    }

    pub fn label(&self) -> String {
        let name = self
            .display_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("pod");
        match self.route_id.as_deref() {
            Some(route) if !route.is_empty() => {
                format!("{} (pod_id={} route_id={})", name, self.pod_id, route)
            }
            _ => format!("{} (pod_id={})", name, self.pod_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PodSelectorError {
    NoPods,
    NotFound {
        selector: String,
    },
    Ambiguous {
        selector: String,
        candidates: Vec<ResolvedPodTarget>,
    },
}

impl fmt::Display for PodSelectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PodSelectorError::NoPods => write!(f, "No workspace yet. Run: tytus connect"),
            PodSelectorError::NotFound { selector } => {
                write!(
                    f,
                    "Pod selector {:?} did not match any active route",
                    selector
                )
            }
            PodSelectorError::Ambiguous {
                selector,
                candidates,
            } => {
                writeln!(
                    f,
                    "Ambiguous pod selector {:?} matched {} routes. Use display name or route_id:",
                    selector,
                    candidates.len()
                )?;
                for c in candidates {
                    let name = c.display_name.as_deref().unwrap_or("<unnamed>");
                    let route = c.route_id.as_deref().unwrap_or("<missing>");
                    writeln!(
                        f,
                        "- {}: pod_id={} route_id={} agent={}",
                        name,
                        c.pod_id,
                        route,
                        c.agent_label()
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for PodSelectorError {}

pub fn resolve_pod_selector(
    selector: Option<&str>,
    state: &CliState,
) -> Result<ResolvedPodTarget, PodSelectorError> {
    resolve_pod_selector_from_pods(selector, &state.pods)
}

pub fn resolve_pod_selector_from_pods(
    selector: Option<&str>,
    pods: &[PodEntry],
) -> Result<ResolvedPodTarget, PodSelectorError> {
    if pods.is_empty() {
        return Err(PodSelectorError::NoPods);
    }

    let Some(raw_selector) = selector.map(str::trim).filter(|s| !s.is_empty()) else {
        if pods.len() == 1 {
            return Ok(ResolvedPodTarget::from_pod(&pods[0]));
        }
        let tunneled: Vec<&PodEntry> = pods
            .iter()
            .filter(|p| p.tunnel_iface.as_deref().is_some_and(|s| !s.is_empty()))
            .collect();
        if tunneled.len() == 1 {
            return Ok(ResolvedPodTarget::from_pod(tunneled[0]));
        }
        return Err(PodSelectorError::Ambiguous {
            selector: "<default>".to_string(),
            candidates: pods.iter().map(ResolvedPodTarget::from_pod).collect(),
        });
    };

    let route_matches: Vec<&PodEntry> = pods
        .iter()
        .filter(|p| p.route_id.as_deref() == Some(raw_selector))
        .collect();
    if let Some(resolved) = unique_or_ambiguous(raw_selector, route_matches)? {
        return Ok(resolved);
    }

    if raw_selector.chars().all(|c| c.is_ascii_digit()) {
        let normalized_pod_id = if raw_selector.len() < 2 {
            format!("{:0>2}", raw_selector)
        } else {
            raw_selector.to_string()
        };
        let pod_matches: Vec<&PodEntry> = pods
            .iter()
            .filter(|p| p.pod_id == raw_selector || p.pod_id == normalized_pod_id)
            .collect();
        return unique_or_ambiguous(raw_selector, pod_matches)?.ok_or_else(|| {
            PodSelectorError::NotFound {
                selector: raw_selector.to_string(),
            }
        });
    }

    let normalized_name = normalize_display_name(raw_selector);
    let name_matches: Vec<&PodEntry> = pods
        .iter()
        .filter(|p| {
            p.display_name
                .as_deref()
                .map(normalize_display_name)
                .as_deref()
                == Some(normalized_name.as_str())
        })
        .collect();

    unique_or_ambiguous(raw_selector, name_matches)?.ok_or_else(|| PodSelectorError::NotFound {
        selector: raw_selector.to_string(),
    })
}

fn unique_or_ambiguous(
    selector: &str,
    matches: Vec<&PodEntry>,
) -> Result<Option<ResolvedPodTarget>, PodSelectorError> {
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(ResolvedPodTarget::from_pod(matches[0]))),
        _ => Err(PodSelectorError::Ambiguous {
            selector: selector.to_string(),
            candidates: matches
                .into_iter()
                .map(ResolvedPodTarget::from_pod)
                .collect(),
        }),
    }
}

fn normalize_display_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pod(pod_id: &str, route_id: &str, display_name: &str, agent_type: &str) -> PodEntry {
        PodEntry {
            pod_id: pod_id.to_string(),
            route_id: Some(route_id.to_string()),
            display_name: Some(display_name.to_string()),
            agent_type: Some(agent_type.to_string()),
            ..Default::default()
        }
    }

    fn fixture() -> Vec<PodEntry> {
        vec![
            pod("01", "0e0ah755r3", "Lisa", "nemoclaw"),
            pod("01", "eb2qvn3t4s", "Claus", "nemoclaw"),
            pod("01", "12gy79s7g0", "Hermie", "hermes"),
            pod("02", "t3n7s69day", "", "none"),
        ]
    }

    #[test]
    fn resolves_by_display_name() {
        let pods = fixture();
        let target = resolve_pod_selector_from_pods(Some(" lisa "), &pods).unwrap();
        assert_eq!(target.route_id.as_deref(), Some("0e0ah755r3"));
        assert_eq!(target.display_name.as_deref(), Some("Lisa"));
    }

    #[test]
    fn resolves_by_route_id_first() {
        let pods = fixture();
        let target = resolve_pod_selector_from_pods(Some("eb2qvn3t4s"), &pods).unwrap();
        assert_eq!(target.display_name.as_deref(), Some("Claus"));
    }

    #[test]
    fn duplicate_pod_id_is_ambiguous() {
        let pods = fixture();
        let err = resolve_pod_selector_from_pods(Some("01"), &pods).unwrap_err();
        match err {
            PodSelectorError::Ambiguous {
                selector,
                candidates,
            } => {
                assert_eq!(selector, "01");
                assert_eq!(candidates.len(), 3);
                assert_eq!(candidates[0].display_name.as_deref(), Some("Lisa"));
                assert_eq!(candidates[1].display_name.as_deref(), Some("Claus"));
                assert_eq!(candidates[2].display_name.as_deref(), Some("Hermie"));
            }
            other => panic!("expected ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_display_name_is_ambiguous() {
        let pods = vec![
            pod("01", "0e0ah755r3", "Lisa", "nemoclaw"),
            pod("02", "eb2qvn3t4s", "Lisa", "nemoclaw"),
        ];
        let err = resolve_pod_selector_from_pods(Some("Lisa"), &pods).unwrap_err();
        assert!(matches!(err, PodSelectorError::Ambiguous { .. }));
    }

    #[test]
    fn numeric_display_name_does_not_bypass_pod_id_ambiguity() {
        let mut pods = fixture();
        pods.push(pod("03", "aaaaaaaaaa", "01", "nemoclaw"));
        let err = resolve_pod_selector_from_pods(Some("01"), &pods).unwrap_err();
        match err {
            PodSelectorError::Ambiguous { candidates, .. } => {
                assert_eq!(candidates.len(), 3);
                assert!(candidates.iter().all(|c| c.pod_id == "01"));
            }
            other => panic!("expected pod-id ambiguity, got {other:?}"),
        }
    }

    #[test]
    fn unique_pod_id_still_resolves() {
        let pods = fixture();
        let target = resolve_pod_selector_from_pods(Some("02"), &pods).unwrap();
        assert_eq!(target.pod_id, "02");
        assert_eq!(target.route_id.as_deref(), Some("t3n7s69day"));
    }

    #[test]
    fn default_selector_fails_closed_when_multiple_routes_exist() {
        let pods = fixture();
        let err = resolve_pod_selector_from_pods(None, &pods).unwrap_err();
        assert!(matches!(err, PodSelectorError::Ambiguous { .. }));
    }
}
