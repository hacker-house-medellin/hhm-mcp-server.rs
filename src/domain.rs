//! Product-owned, deterministic domain inventory and planning.

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

pub const ORGANIZATION: &str = "hacker-house-medellin";
pub const REPOSITORY: &str = "hhm-mcp-server.rs";
pub const DOMAIN_SUMMARY: &str =
    "Read-only MCP diagnostics for applications, residents, stays, rooms, events, and projects";
pub const UNIT_LABEL: &str = "residency units";

const REPOSITORIES: [(&str, &str); 5] = [
    (
        "hhm-api",
        "applications, residents, stays, rooms, events, and projects API",
    ),
    (
        "hhm-interfaces",
        "member, stay, room, event, and project contracts",
    ),
    (
        "hhm-sync",
        "offline-first residency and project synchronization",
    ),
    ("hhm-clients", "typed client SDKs"),
    (
        "hhm-infra",
        "Kubernetes and bounded Cloudflare edge infrastructure",
    ),
];

const READINESS_VARIABLES: [&str; 4] = [
    "SUPABASE_URL",
    "SHARED_AUTH_BASE_URL",
    "CLOUDFLARE_API_TOKEN",
    "FIDUCIA_TOKEN",
];

const DOMAIN_NOTES: [&str; 3] = [
    "MCP never approves an application, assigns a room, or changes a stay.",
    "Resident and applicant identities are excluded from tools and telemetry.",
    "Capacity calculations are advisory and require operator authorization elsewhere.",
];

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Workload {
    ApplicationReview,
    RoomCapacity,
    CommunityEvent,
}

impl Workload {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApplicationReview => "application_review",
            Self::RoomCapacity => "room_capacity",
            Self::CommunityEvent => "community_event",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PlanInput {
    /// Closed product workload; arbitrary commands and URLs are impossible.
    pub workload: Workload,
    /// Bounded number of residency units to evaluate.
    #[schemars(range(min = 1, max = 100_000))]
    pub units: u32,
    /// Redundancy multiplier. Zero is valid for a single planned lane.
    #[schemars(range(min = 0, max = 5))]
    pub redundancy: u8,
}

/// Static ownership only; this is not evidence of provider readiness.
#[must_use]
pub fn authority_map() -> Value {
    json!({
        "contracts": "hacker-house-medellin/hhm-interfaces",
        "domain": "hacker-house-medellin/hhm-lib-core",
        "persistence": "hacker-house-medellin/hhm-orm-core",
        "deployment": "hacker-house-medellin/hhm-infra",
        "readOnly": true,
        "runtimeVerified": false,
        "mutationExposed": false
    })
}

#[must_use]
pub fn fleet_map() -> Value {
    let repositories = REPOSITORIES
        .into_iter()
        .map(|(name, role)| json!({"name": name, "role": role}))
        .collect::<Vec<_>>();
    json!({
        "organization": ORGANIZATION,
        "repository": REPOSITORY,
        "summary": DOMAIN_SUMMARY,
        "repositories": repositories,
        "readOnly": true
    })
}

/// Build a bounded deterministic capacity plan.
///
/// # Errors
///
/// Returns an error when numeric fields are outside their documented
/// bounds or the derived capacity would overflow.
pub fn plan(input: PlanInput) -> Result<Value, String> {
    if !(1..=100_000).contains(&input.units) {
        return Err("units must be between 1 and 100000".to_string());
    }
    if input.redundancy > 5 {
        return Err("redundancy must be between 0 and 5".to_string());
    }
    let lanes = u32::from(input.redundancy) + 1;
    let planned_capacity = input
        .units
        .checked_mul(lanes)
        .ok_or_else(|| "planned capacity overflowed".to_string())?;
    Ok(json!({
        "workload": input.workload.as_str(),
        "units": input.units,
        "unitLabel": UNIT_LABEL,
        "redundancy": input.redundancy,
        "lanes": lanes,
        "plannedCapacity": planned_capacity,
        "advisoryOnly": true,
        "executed": false,
        "mutated": false
    }))
}

#[must_use]
pub fn runtime_readiness() -> Value {
    let variables = READINESS_VARIABLES
        .into_iter()
        .map(|name| {
            let value = std::env::var_os(name);
            json!({"name": name, "configured": configured_value(value.as_deref())})
        })
        .collect::<Vec<_>>();
    json!({
        "configuration": variables,
        "valuesExposed": false,
        "networkChecked": false,
        "authenticated": false
    })
}

// Configuration presence is only meaningful for non-blank UTF-8 values.
// Never place the value (including invalid Unicode) in tool output or telemetry.
fn configured_value(value: Option<&std::ffi::OsStr>) -> bool {
    value
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|value| !value.trim().is_empty())
}

#[must_use]
pub fn safety_boundary() -> Value {
    json!({
        "notes": DOMAIN_NOTES,
        "argumentsLogged": false,
        "resultsLogged": false,
        "secretsLogged": false,
        "mutationTools": false,
        "arbitraryUrlsAccepted": false,
        "arbitraryCommandsAccepted": false
    })
}

#[must_use]
pub const fn workload_names() -> [&'static str; 3] {
    ["application_review", "room_capacity", "community_event"]
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn plan_is_bounded_and_deterministic() {
        let input: PlanInput = serde_json::from_value(json!({
            "workload": workload_names()[0],
            "units": 7,
            "redundancy": 2
        }))
        .expect("closed valid plan");
        let output = plan(input).expect("valid plan");
        assert_eq!(output["plannedCapacity"], 21);
        assert_eq!(output["executed"], false);
        assert_eq!(output["mutated"], false);
    }

    #[test]
    fn plan_rejects_unknown_fields_and_runtime_outliers() {
        assert!(
            serde_json::from_value::<PlanInput>(json!({
                "workload": workload_names()[0],
                "units": 1,
                "redundancy": 0,
                "command": "forbidden"
            }))
            .is_err()
        );
        let too_large: PlanInput = serde_json::from_value(json!({
            "workload": workload_names()[0],
            "units": 100_001,
            "redundancy": 0
        }))
        .expect("serde intentionally delegates numeric bounds");
        assert!(plan(too_large).is_err());
    }

    #[test]
    fn static_authority_map_is_read_only_and_not_a_live_readiness_claim() {
        let map = authority_map();
        assert_eq!(map["contracts"], "hacker-house-medellin/hhm-interfaces");
        assert_eq!(map["persistence"], "hacker-house-medellin/hhm-orm-core");
        assert_eq!(map["readOnly"], true);
        assert_eq!(map["runtimeVerified"], false);
        assert_eq!(map["mutationExposed"], false);
    }

    #[test]
    fn readiness_rejects_blank_or_missing_configuration_without_exposing_values() {
        use std::ffi::OsStr;

        assert!(!configured_value(None));
        assert!(!configured_value(Some(OsStr::new(""))));
        assert!(!configured_value(Some(OsStr::new(" \t\n"))));
        assert!(configured_value(Some(OsStr::new("opaque-secret"))));
        assert!(configured_value(Some(OsStr::new("https://example.test"))));
    }

    #[cfg(unix)]
    #[test]
    fn readiness_rejects_non_utf8_configuration() {
        use std::os::unix::ffi::OsStrExt;

        assert!(!configured_value(Some(std::ffi::OsStr::from_bytes(&[
            0xff, 0xfe
        ]))));
    }

    #[test]
    fn readiness_discloses_presence_only() {
        let value = runtime_readiness();
        assert_eq!(value["valuesExposed"], false);
        assert_eq!(value["networkChecked"], false);
        assert_eq!(value["authenticated"], false);
    }
}
