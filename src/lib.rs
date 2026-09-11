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
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

// ─── Steerwishes ────────────────────────────────────────────────────
// Mirrors images/godon-api/openapi.yml schemas (Steerwish, SteerwishSummary,
// SteerwishCreate, SteerwishBand, SteerwishLimits, SteerwishEvent).

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerwishBand {
    pub lo: f64,
    pub hi: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerwishLimits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(rename = "maxChange", default, skip_serializing_if = "Option::is_none")]
    pub max_change: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteerwishCreate {
    pub outcome: String,
    pub band: SteerwishBand,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<SteerwishLimits>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regime: Option<String>,
}

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
    pub outcome: String,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Steerwish {
    pub id: String,
    pub outcome: String,
    pub band: SteerwishBand,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<SteerwishLimits>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub budget: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regime: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<SteerwishEvent>>,
}

#[cfg(test)]
mod steerwish_tests {
    use super::*;

    // Stub cases, mirroring the openapi examples
    // (images/godon-api/openapi.yml, schemas Steerwish*).

    #[test]
    fn steerwish_create_parses_from_yaml() {
        let yaml = r#"
outcome: chainend.shift
band:
  lo: -0.14
  hi: -0.06
  target: -0.10
limits:
  exclude: []
  maxChange: 0.5
budget: 2
regime: standing
"#;
        let wish: SteerwishCreate = serde_yaml::from_str(yaml).expect("parse");
        assert_eq!(wish.outcome, "chainend.shift");
        assert_eq!(wish.band.lo, -0.14);
        assert_eq!(wish.band.hi, -0.06);
        assert_eq!(wish.band.target, Some(-0.10));
        assert_eq!(wish.limits.as_ref().unwrap().max_change, Some(0.5));
        assert_eq!(wish.budget, Some(2));
        assert_eq!(wish.regime.as_deref(), Some("standing"));
    }

    #[test]
    fn steerwish_create_minimal_omits_optionals() {
        let wish = SteerwishCreate {
            outcome: "chainend.shift".into(),
            band: SteerwishBand { lo: -0.14, hi: -0.06, target: None },
            limits: None,
            budget: None,
            regime: None,
        };
        let json = serde_json::to_value(&wish).unwrap();
        assert_eq!(json["outcome"], "chainend.shift");
        assert_eq!(json["band"]["lo"], -0.14);
        assert!(json.get("budget").is_none(), "omitted budget must not serialize");
        assert!(json.get("regime").is_none());
        assert!(json.get("limits").is_none());
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
