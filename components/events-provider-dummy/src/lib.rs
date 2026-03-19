//! Dummy event provider component.
//!
//! Provides a simple dummy event provider for testing and development.

#![deny(unsafe_op_in_unsafe_fn)]

mod bindings {
    wit_bindgen::generate!({
        path: "wit/events-provider-dummy",
        world: "component-v0-v6-v0",
        generate_all
    });
}

mod describe;

use provider_common::component_v0_6::{canonical_cbor_bytes, decode_cbor};
use provider_common::helpers::{
    cbor_json_invoke_bridge, existing_config_from_answers, json_bytes, schema_core_describe,
    schema_core_healthcheck, schema_core_validate_config, string_or_default,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

pub(crate) const PROVIDER_ID: &str = "events-provider-dummy";
pub(crate) const WORLD_ID: &str = "greentic:component/component@0.6.1";

use describe::{
    DEFAULT_KEYS, I18N_KEYS, I18N_PAIRS, SETUP_QUESTIONS, build_describe_payload, build_qa_spec,
};

/// Provider configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self { enabled: true }
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

/// Dummy input.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct DummyInput {
    #[serde(default)]
    config: ProviderConfig,
    #[serde(default)]
    payload: Value,
}

/// Apply answers to produce a config.
fn apply_answers_impl(answers: &Value) -> ProviderConfigOut {
    // Check for existing config to merge with
    let base: ProviderConfig = existing_config_from_answers(answers).unwrap_or_default();

    let enabled_str = string_or_default(
        answers,
        "enabled",
        if base.enabled { "true" } else { "false" },
    );
    let enabled = matches!(
        enabled_str.to_lowercase().as_str(),
        "true" | "yes" | "1" | "on"
    );

    let config = ProviderConfig { enabled };

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

fn stable_receipt_id(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &bytes).to_string()
}

fn handle_publish(input: &DummyInput) -> Value {
    let receipt_id = stable_receipt_id(&input.payload);

    json!({
        "ok": true,
        "receipt_id": receipt_id,
        "status": "published",
    })
}

fn handle_echo(input: &DummyInput) -> Value {
    json!({
        "ok": true,
        "echo": input.payload,
    })
}

fn dispatch(op: &str, input_json: &[u8]) -> Vec<u8> {
    let parsed: Result<DummyInput, _> = serde_json::from_slice(input_json);
    match op {
        "publish" => match parsed {
            Ok(input) => {
                if !input.config.enabled {
                    return json_bytes(&json!({"ok": false, "error": "provider disabled"}));
                }
                let result = handle_publish(&input);
                json_bytes(&result)
            }
            Err(err) => json_bytes(&json!({"ok": false, "error": format!("invalid input: {err}")})),
        },
        "echo" => match parsed {
            Ok(input) => {
                if !input.config.enabled {
                    return json_bytes(&json!({"ok": false, "error": "provider disabled"}));
                }
                let result = handle_echo(&input);
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
            "dummy",
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

    fn sample_input() -> DummyInput {
        DummyInput {
            config: ProviderConfig { enabled: true },
            payload: json!({"message": "hello"}),
        }
    }

    #[test]
    fn receipt_is_deterministic() {
        let input = sample_input();
        let id1 = stable_receipt_id(&input.payload);
        let id2 = stable_receipt_id(&input.payload);
        assert_eq!(id1, id2);
    }

    #[test]
    fn publish_returns_receipt() {
        let input = sample_input();
        let result = handle_publish(&input);
        assert_eq!(result["ok"], true);
        assert!(result["receipt_id"].is_string());
        assert_eq!(result["status"], "published");
    }

    #[test]
    fn echo_returns_payload() {
        let input = sample_input();
        let result = handle_echo(&input);
        assert_eq!(result["ok"], true);
        assert_eq!(result["echo"]["message"], "hello");
    }

    #[test]
    fn apply_answers_produces_config() {
        let answers = json!({
            "enabled": "true",
        });
        let out = apply_answers_impl(&answers);
        assert!(out.ok);
        let cfg = out.config.expect("config");
        assert!(cfg.enabled);
    }

    #[test]
    fn apply_answers_defaults_to_enabled() {
        let answers = json!({});
        let out = apply_answers_impl(&answers);
        assert!(out.ok);
        let cfg = out.config.expect("config");
        assert!(cfg.enabled);
    }

    #[test]
    fn apply_answers_can_disable() {
        let answers = json!({
            "enabled": "false",
        });
        let out = apply_answers_impl(&answers);
        assert!(out.ok);
        let cfg = out.config.expect("config");
        assert!(!cfg.enabled);
    }
}
