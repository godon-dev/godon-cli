pub mod client;

pub use client::GodonClient;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemtenderSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Systemtender {
    pub id: String,
    pub name: String,
    pub status: String,
    pub config: serde_json::Value,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// Reason the last deletion attempt failed; present only while
    /// status is deletion-failed (re-DELETE is the retry).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deletion_reason: Option<String>,
    /// Reason the create executor failed; present only while status
    /// is create-failed (DELETE clears the row, then retry create).
    /// Mirrors deletion_reason on the creation axis
    /// (designs/2026-10-10-async-create-contract.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creation_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemtenderCreateRequest {
    pub name: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemtenderUpdateRequest {
    pub config: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub name: String,
    #[serde(rename = "credentialType")]
    pub credential_type: String,
    pub description: Option<String>,
    #[serde(rename = "windmillVariable")]
    pub windmill_variable: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub id: String,
    pub name: String,
    #[serde(rename = "targetType")]
    pub target_type: String,
    pub spec: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "lastUsedAt")]
    pub last_used_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub hostname: String,
    pub port: u16,
    pub api_version: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            hostname: "localhost".to_string(),
            port: 8080,
            api_version: "v0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// HTTP status of the response, when one arrived. The async
    /// contract translates expectations to exit codes: the HTTP
    /// status says what happened to the REQUEST (409 = name taken,
    /// 404 = gone), the row state says what happened to the
    /// systemtender (designs/2026-10-10).
    #[serde(skip)]
    pub status: Option<u16>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            status: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
            status: None,
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = Some(status);
        self
    }
}

// ─── Async contract exit codes (designs/2026-10-10) ────────────────
// The HTTP status never carries the expectation verdict — the state
// does; the CLI translates unmet expectations to exit codes so
// scripts can branch without parsing prose.
pub const EXIT_OK: i32 = 0;
/// Transport/parse error (the pre-existing behavior of write_error).
pub const EXIT_ERROR: i32 = 1;
/// Create hit a name that already exists, in any state — including
/// one still `creating` or one that `create-failed` (HTTP 409).
pub const EXIT_NAME_TAKEN: i32 = 2;
/// `create --wait` ended with the expectation unmet: the row reads
/// create-failed, or is still `creating` at the timeout bound.
pub const EXIT_CREATE_UNMET: i32 = 3;
/// `purge --wait` ended with the row still present at the timeout
/// bound (poll GET again, or read the state + deletion_reason).
pub const EXIT_PURGE_UNMET: i32 = 4;

/// Terminal-for-create from the client's side: the row is live. The
/// first heartbeat may refine `active` into `running` — both mean
/// the create landed. `presumed_dead` does not: the workers died.
pub fn create_wait_met(status: &str) -> bool {
    matches!(status, "active" | "running" | "finished")
}

// ─── Steerwishes ────────────────────────────────────────────────────
// Mirrors images/godon-api/openapi.yml schemas (Steerwish, SteerwishSummary,
// SteerwishEvent). Wish-shape freedom (2026-09-21): the wish body is the
// declarer's own - the CLI carries it verbatim and the controller validates
// (the door). Only the lifecycle envelope is typed here.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerwishEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerwishSummary {
    pub id: String,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(flatten)]
    pub body: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Steerwish {
    pub id: String,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<SteerwishEvent>>,
    #[serde(flatten)]
    pub body: serde_json::Map<String, serde_json::Value>,
}

#[cfg(test)]
mod steerwish_tests {
    use super::*;

    // Stub cases, mirroring the openapi examples
    // (images/godon-api/openapi.yml, schemas Steerwish*).

    #[test]
    fn steerwish_yaml_passthrough_verbatim() {
        // wish-shape freedom: unknown/claim-shaped fields survive the
        // YAML -> JSON hop untouched - the controller validates (the door)
        let yaml = r#"
outcome: chainend.shift
band:
  lo: -0.14
  hi: -0.06
  target: -0.10
claims:
  - outcome: latency(x)
    direction: minimize
"#;
        let parsed: serde_yaml::Value = serde_yaml::from_str(yaml).expect("parse");
        let json = serde_json::to_value(&parsed).unwrap();
        assert_eq!(json["outcome"], "chainend.shift");
        assert_eq!(json["band"]["lo"], -0.14);
        assert_eq!(json["claims"][0]["direction"], "minimize");
    }

    #[test]
    fn steerwish_envelope_keeps_body_verbatim() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "outcome": "chainend.shift",
            "band": {"lo": -0.14, "hi": -0.06},
            "state": "declared",
            "createdAt": "2026-09-10T10:30:00Z"
        }"#;
        let wish: Steerwish = serde_json::from_str(json).expect("parse");
        assert_eq!(wish.state, "declared");
        assert_eq!(wish.body.get("outcome").and_then(|v| v.as_str()), Some("chainend.shift"));
        assert_eq!(wish.body["band"]["lo"], -0.14);
    }

    #[test]
    fn steerwish_with_events_parses() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "outcome": "chainend.shift",
            "band": {"lo": -0.14, "hi": -0.06, "target": -0.10},
            "state": "declared",
            "createdAt": "2026-09-10T10:30:00Z",
            "regime": "standing",
            "events": [
                {"type": "declared", "at": "2026-09-10T10:30:00Z"},
                {"type": "refused", "at": "2026-09-10T10:31:00Z",
                 "detail": "target outside measured range"}
            ]
        }"#;
        let wish: Steerwish = serde_json::from_str(json).expect("parse");
        assert_eq!(wish.state, "declared");
        let events = wish.events.as_ref().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "declared");
        assert_eq!(events[1].kind, "refused");
        assert_eq!(
            events[1].detail.as_deref(),
            Some("target outside measured range")
        );
    }

    #[test]
    fn steerwish_summary_parses_without_band() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "outcome": "chainend.shift",
            "state": "planned",
            "createdAt": "2026-09-10T10:30:00Z"
        }"#;
        let summary: SteerwishSummary = serde_json::from_str(json).expect("parse");
        assert_eq!(summary.state, "planned");
        assert_eq!(summary.created_at, "2026-09-10T10:30:00Z");
    }
}
