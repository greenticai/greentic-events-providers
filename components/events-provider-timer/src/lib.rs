//! Timer event provider component.
//!
//! Provides timer/scheduler functionality for event processing.

#![deny(unsafe_op_in_unsafe_fn)]

mod bindings {
    wit_bindgen::generate!({
        path: "wit/events-provider-timer",
        world: "component-v0-v6-v0",
        generate_all
    });
}

mod describe;

use chrono::Utc;
use provider_common::component_v0_6::{canonical_cbor_bytes, decode_cbor};
use provider_common::helpers::{
    cbor_json_invoke_bridge, existing_config_from_answers, json_bytes, optional_string_from,
    schema_core_describe, schema_core_healthcheck, schema_core_validate_config, string_or_default,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

pub(crate) const PROVIDER_ID: &str = "events-provider-timer";
pub(crate) const WORLD_ID: &str = "greentic:component/component@0.6.1";

use describe::{
    DEFAULT_KEYS, I18N_KEYS, I18N_PAIRS, SETUP_QUESTIONS, build_describe_payload, build_qa_spec,
};

/// Provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default)]
    pub default_delay_seconds: Option<u64>,
    #[serde(default)]
    pub persistence_key_prefix: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_timezone() -> String {
    "UTC".into()
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timezone: default_timezone(),
            default_delay_seconds: Some(30),
            persistence_key_prefix: Some("events/timer/scheduled".into()),
        }
    }
}

/// Output configuration from apply-answers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfigOut {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ProviderConfig>,
}

/// Timer tick input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TickInput {
    config: ProviderConfig,
    #[serde(default)]
    event: Value,
    #[serde(default)]
    handler_id: Option<String>,
    #[serde(default)]
    tenant: Option<String>,
    #[serde(default)]
    team: Option<String>,
    #[serde(default)]
    correlation_id: Option<String>,
}

/// Scheduled entry for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduledEntry {
    event: Value,
    queued_at: String,
    timezone: String,
    default_delay_seconds: Option<u64>,
}

/// Apply answers to produce a config.
fn apply_answers_impl(answers: &Value) -> ProviderConfigOut {
    // Check for existing config to merge with
    let base: ProviderConfig = existing_config_from_answers(answers).unwrap_or_default();

    let enabled_str = string_or_default(answers, "enabled", "true");
    let enabled = matches!(
        enabled_str.to_lowercase().as_str(),
        "true" | "yes" | "1" | "on"
    );

    let timezone = string_or_default(answers, "timezone", base.timezone.as_str());

    let default_delay_seconds = optional_string_from(answers, "default_delay_seconds")
        .and_then(|s| s.parse::<u64>().ok())
        .or(base.default_delay_seconds);

    let persistence_key_prefix =
        optional_string_from(answers, "persistence_key_prefix").or(base.persistence_key_prefix);

    let config = ProviderConfig {
        enabled,
        timezone,
        default_delay_seconds,
        persistence_key_prefix,
    };

    ProviderConfigOut {
        ok: true,
        error: None,
        config: Some(config),
    }
}

fn apply_answers_bridge(_mode: &str, answers_cbor: Vec<u8>) -> Vec<u8> {
    let answers: Value = match decode_cbor(&answers_cbor) {
        Ok(val) => val,
        Err(err) => {
            return canonical_cbor_bytes(&ProviderConfigOut {
                ok: false,
                error: Some(format!("invalid cbor: {err}")),
                config: None,
            });
        }
    };
    let out = apply_answers_impl(&answers);
    canonical_cbor_bytes(&out)
}

fn state_key(config: &ProviderConfig, receipt_id: &str) -> String {
    let prefix = config
        .persistence_key_prefix
        .as_deref()
        .unwrap_or("events/timer/scheduled");
    format!("{prefix}/{receipt_id}.json")
}

fn stable_receipt_id(event: &Value) -> String {
    let bytes = serde_json::to_vec(event).unwrap_or_default();
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &bytes).to_string()
}

fn handle_timer_tick(input: &TickInput) -> Value {
    let receipt_id = stable_receipt_id(&input.event);
    let key = state_key(&input.config, &receipt_id);

    // Create scheduled entry
    let _entry = ScheduledEntry {
        event: input.event.clone(),
        queued_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        timezone: input.config.timezone.clone(),
        default_delay_seconds: input.config.default_delay_seconds,
    };

    // Note: In WASM target, we would persist via state_store
    // For now, we just generate the output

    let now = Utc::now().to_rfc3339();

    let emitted_event = json!({
        "event_id": receipt_id,
        "event_type": "timer.tick",
        "occurred_at": now,
        "source": {
            "domain": "events",
            "provider": "events.timer",
            "handler_id": input.handler_id.clone().unwrap_or_else(|| "default".to_string()),
        },
        "scope": {
            "tenant": input.tenant.clone().unwrap_or_else(|| "default".to_string()),
            "team": input.team,
            "correlation_id": input.correlation_id,
        },
        "payload": input.event,
    });

    json!({
        "ok": true,
        "receipt_id": receipt_id,
        "status": "queued",
        "state_key": key,
        "emitted_events": [emitted_event],
    })
}

fn dispatch(op: &str, input_json: &[u8]) -> Vec<u8> {
    let parsed: Result<TickInput, _> = serde_json::from_slice(input_json);
    match op {
        "timer_tick" | "publish" => match parsed {
            Ok(input) => {
                if !input.config.enabled {
                    return json_bytes(&json!({"ok": false, "error": "provider disabled"}));
                }
                let result = handle_timer_tick(&input);
                json_bytes(&result)
            }
            Err(err) => json_bytes(&json!({"ok": false, "error": format!("invalid input: {err}")})),
        },
        _ => json_bytes(&json!({"ok": false, "error": format!("unknown operation: {op}")})),
    }
}

struct Component;

impl bindings::exports::greentic::component::descriptor::Guest for Component {
    fn describe() -> Vec<u8> {
        schema_core_describe(&build_describe_payload())
    }
}

impl bindings::exports::greentic::component::runtime::Guest for Component {
    fn invoke(op: String, input_cbor: Vec<u8>) -> Vec<u8> {
        cbor_json_invoke_bridge(&op, &input_cbor, None, dispatch)
    }
}

impl bindings::exports::greentic::component::qa::Guest for Component {
    fn qa_spec(mode: bindings::exports::greentic::component::qa::Mode) -> Vec<u8> {
        let spec = build_qa_spec(mode);
        canonical_cbor_bytes(&spec)
    }

    fn apply_answers(
        _mode: bindings::exports::greentic::component::qa::Mode,
        answers_cbor: Vec<u8>,
    ) -> Vec<u8> {
        let answers: Value = match decode_cbor(&answers_cbor) {
            Ok(val) => val,
            Err(err) => {
                return canonical_cbor_bytes(&ProviderConfigOut {
                    ok: false,
                    error: Some(format!("invalid cbor: {err}")),
                    config: None,
                });
            }
        };
        let out = apply_answers_impl(&answers);
        canonical_cbor_bytes(&out)
    }
}

impl bindings::exports::greentic::component::component_i18n::Guest for Component {
    fn i18n_keys() -> Vec<String> {
        I18N_KEYS.iter().map(|s| (*s).to_string()).collect()
    }

    fn i18n_bundle(locale: String) -> Vec<u8> {
        describe::i18n_bundle(locale)
    }
}

impl bindings::exports::greentic::provider_schema_core::schema_core_api::Guest for Component {
    fn describe() -> Vec<u8> {
        schema_core_describe(&build_describe_payload())
    }

    fn validate_config(_config_json: Vec<u8>) -> Vec<u8> {
        schema_core_validate_config()
    }

    fn healthcheck() -> Vec<u8> {
        schema_core_healthcheck()
    }

    fn invoke(op: String, input_json: Vec<u8>) -> Vec<u8> {
        // Handle QA ops via JSON (for operator compatibility)
        if let Some(result) = provider_common::qa_invoke_bridge::dispatch_qa_ops_with_i18n(
            &op,
            &input_json,
            "timer",
            SETUP_QUESTIONS,
            DEFAULT_KEYS,
            I18N_KEYS,
            I18N_PAIRS,
            apply_answers_bridge,
        ) {
            return result;
        }
        dispatch(&op, &input_json)
    }
}

bindings::export!(Component with_types_in bindings);

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_input() -> TickInput {
        TickInput {
            config: ProviderConfig {
                enabled: true,
                timezone: "UTC".into(),
                default_delay_seconds: Some(30),
                persistence_key_prefix: Some("events/timer/scheduled".into()),
            },
            event: json!({"kind": "reminder", "id": 1}),
            handler_id: Some("nightly-reminder".into()),
            tenant: Some("tenant-a".into()),
            team: Some("team-1".into()),
            correlation_id: Some("corr-123".into()),
        }
    }

    #[test]
    fn receipt_is_deterministic() {
        let input = sample_input();
        let id1 = stable_receipt_id(&input.event);
        let id2 = stable_receipt_id(&input.event);
        assert_eq!(id1, id2);
    }

    #[test]
    fn timer_tick_returns_envelope() {
        let input = sample_input();
        let result = handle_timer_tick(&input);
        assert_eq!(result["ok"], true);
        assert!(result["receipt_id"].is_string());
        assert!(result["emitted_events"].is_array());
    }

    #[test]
    fn apply_answers_produces_config() {
        let answers = json!({
            "enabled": "true",
            "timezone": "America/New_York",
            "default_delay_seconds": "60",
        });
        let out = apply_answers_impl(&answers);
        assert!(out.ok);
        let cfg = out.config.expect("config");
        assert!(cfg.enabled);
        assert_eq!(cfg.timezone, "America/New_York");
        assert_eq!(cfg.default_delay_seconds, Some(60));
    }

    #[test]
    fn apply_answers_uses_defaults() {
        let answers = json!({});
        let out = apply_answers_impl(&answers);
        assert!(out.ok);
        let cfg = out.config.expect("config");
        assert!(cfg.enabled);
        assert_eq!(cfg.timezone, "UTC");
    }
}
