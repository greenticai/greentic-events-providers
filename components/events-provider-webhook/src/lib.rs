//! Webhook event provider component.
//!
//! Implements the v0.6.0 QA contract with setup/upgrade/remove modes.

use chrono::Utc;
use provider_common::component_v0_6::{canonical_cbor_bytes, decode_cbor};
use provider_common::helpers::{
    cbor_json_invoke_bridge, existing_config_from_answers, json_bytes, optional_string_from,
    schema_core_describe, schema_core_healthcheck, schema_core_validate_config, string_or_default,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use uuid::Uuid;

mod bindings {
    wit_bindgen::generate!({
        path: "wit/events-provider-webhook",
        world: "component-v0-v6-v0",
        generate_all
    });
}

mod describe;

pub(crate) const PROVIDER_ID: &str = "events-provider-webhook";
pub(crate) const WORLD_ID: &str = "component-v0-v6-v0";

use describe::{
    DEFAULT_KEYS, I18N_KEYS, I18N_PAIRS, SETUP_QUESTIONS, build_describe_payload, build_qa_spec,
};

// ============================================================================
// Provider configuration
// ============================================================================

/// Provider configuration output from QA apply-answers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderConfigOut {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub target_url: Option<String>,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub auth_token: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

fn default_true() -> bool {
    true
}

fn default_method() -> String {
    "POST".to_string()
}

fn default_config_out() -> ProviderConfigOut {
    ProviderConfigOut {
        enabled: true,
        target_url: None,
        method: "POST".to_string(),
        auth_token: None,
        timeout_ms: Some(5000),
    }
}

fn validate_config_out(config: &ProviderConfigOut) -> Result<(), String> {
    if let Some(target_url) = config.target_url.as_deref()
        && !target_url.starts_with("http://")
        && !target_url.starts_with("https://")
    {
        return Err("target_url must be a valid HTTP/HTTPS URL".to_string());
    }
    Ok(())
}

// ============================================================================
// Input/Output types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IngestInput {
    config: IngestConfig,
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
    #[serde(default)]
    http: Option<Value>,
    #[serde(default)]
    raw: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IngestConfig {
    #[serde(default)]
    target_url: Option<String>,
    #[serde(default = "default_method")]
    method: String,
    #[serde(default)]
    headers: BTreeMap<String, String>,
    #[serde(default)]
    auth: Option<String>,
    #[serde(default)]
    timeout_ms: Option<u64>,
}

// ============================================================================
// Component trait implementations
// ============================================================================

struct Component;

impl bindings::exports::greentic::component::descriptor::Guest for Component {
    fn describe() -> Vec<u8> {
        canonical_cbor_bytes(&build_describe_payload())
    }
}

impl bindings::exports::greentic::component::runtime::Guest for Component {
    fn invoke(op: String, input_cbor: Vec<u8>) -> Vec<u8> {
        cbor_json_invoke_bridge(&op, &input_cbor, Some("ingest_http"), dispatch_json_invoke)
    }
}

impl bindings::exports::greentic::component::qa::Guest for Component {
    fn qa_spec(mode: bindings::exports::greentic::component::qa::Mode) -> Vec<u8> {
        canonical_cbor_bytes(&build_qa_spec(mode))
    }

    fn apply_answers(
        mode: bindings::exports::greentic::component::qa::Mode,
        answers_cbor: Vec<u8>,
    ) -> Vec<u8> {
        apply_answers_impl(mode, answers_cbor)
    }
}

impl bindings::exports::greentic::component::component_i18n::Guest for Component {
    fn i18n_keys() -> Vec<String> {
        I18N_KEYS.iter().map(|k| (*k).to_string()).collect()
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
        if let Some(result) = provider_common::qa_invoke_bridge::dispatch_qa_ops_with_i18n(
            &op,
            &input_json,
            "webhook",
            SETUP_QUESTIONS,
            DEFAULT_KEYS,
            I18N_KEYS,
            I18N_PAIRS,
            apply_answers_bridge,
        ) {
            return result;
        }
        dispatch_json_invoke(&op, &input_json)
    }
}

bindings::export!(Component with_types_in bindings);

// ============================================================================
// Dispatch
// ============================================================================

fn apply_answers_bridge(mode: &str, answers_cbor: Vec<u8>) -> Vec<u8> {
    use bindings::exports::greentic::component::qa::Mode;
    let mode = match mode {
        "setup" => Mode::Setup,
        "upgrade" => Mode::Upgrade,
        "remove" => Mode::Remove,
        _ => Mode::Default,
    };
    apply_answers_impl(mode, answers_cbor)
}

fn dispatch_json_invoke(op: &str, input_json: &[u8]) -> Vec<u8> {
    match op {
        "ingest_http" => handle_ingest_http(input_json),
        "run" | "publish" => handle_publish(input_json),
        other => json_bytes(&json!({"ok": false, "error": format!("unsupported op: {other}")})),
    }
}

// ============================================================================
// QA apply_answers implementation
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApplyAnswersResult {
    ok: bool,
    config: Option<ProviderConfigOut>,
    remove: Option<RemovePlan>,
    diagnostics: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RemovePlan {
    remove_all: bool,
    cleanup: Vec<String>,
}

fn apply_answers_impl(
    mode: bindings::exports::greentic::component::qa::Mode,
    answers_cbor: Vec<u8>,
) -> Vec<u8> {
    use bindings::exports::greentic::component::qa::Mode;

    let answers: Value = match decode_cbor(&answers_cbor) {
        Ok(value) => value,
        Err(err) => {
            return canonical_cbor_bytes(&ApplyAnswersResult {
                ok: false,
                config: None,
                remove: None,
                diagnostics: Vec::new(),
                error: Some(format!("invalid answers cbor: {err}")),
            });
        }
    };

    if mode == Mode::Remove {
        return canonical_cbor_bytes(&ApplyAnswersResult {
            ok: true,
            config: None,
            remove: Some(RemovePlan {
                remove_all: true,
                cleanup: vec![
                    "delete_config_key".to_string(),
                    "delete_provenance_key".to_string(),
                    "delete_provider_state_namespace".to_string(),
                ],
            }),
            diagnostics: Vec::new(),
            error: None,
        });
    }

    let mut merged = existing_config_from_answers(&answers).unwrap_or_else(default_config_out);
    let answer_obj = answers.as_object();
    let has = |key: &str| answer_obj.is_some_and(|obj| obj.contains_key(key));

    if mode == Mode::Setup || mode == Mode::Default {
        merged.enabled = answers
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(merged.enabled);
        merged.target_url = optional_string_from(&answers, "target_url").or(merged.target_url);
        merged.method = string_or_default(&answers, "method", &merged.method);
        if merged.method.trim().is_empty() {
            merged.method = "POST".to_string();
        }
        merged.auth_token =
            optional_string_from(&answers, "auth_token").or(merged.auth_token.clone());
        if let Some(timeout) = answers.get("timeout_ms").and_then(Value::as_u64) {
            merged.timeout_ms = Some(timeout);
        }
    }

    if mode == Mode::Upgrade {
        if has("enabled") {
            merged.enabled = answers
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(merged.enabled);
        }
        if has("target_url") {
            merged.target_url = optional_string_from(&answers, "target_url");
        }
        if has("method") {
            merged.method = string_or_default(&answers, "method", &merged.method);
        }
        if has("auth_token") {
            merged.auth_token = optional_string_from(&answers, "auth_token");
        }
        if has("timeout_ms")
            && let Some(timeout) = answers.get("timeout_ms").and_then(Value::as_u64)
        {
            merged.timeout_ms = Some(timeout);
        }
        if merged.method.trim().is_empty() {
            merged.method = "POST".to_string();
        }
    }

    if let Err(error) = validate_config_out(&merged) {
        return canonical_cbor_bytes(&ApplyAnswersResult {
            ok: false,
            config: None,
            remove: None,
            diagnostics: Vec::new(),
            error: Some(error),
        });
    }

    canonical_cbor_bytes(&ApplyAnswersResult {
        ok: true,
        config: Some(merged),
        remove: None,
        diagnostics: Vec::new(),
        error: None,
    })
}

// ============================================================================
// Operations
// ============================================================================

fn handle_ingest_http(input_json: &[u8]) -> Vec<u8> {
    let parsed: IngestInput = match serde_json::from_slice(input_json) {
        Ok(v) => v,
        Err(e) => {
            return json_bytes(&json!({"ok": false, "error": format!("invalid input: {e}")}));
        }
    };

    let receipt_id = stable_receipt_id(&parsed.event);
    let now = Utc::now().to_rfc3339();

    let mut emitted_event = json!({
        "event_id": receipt_id,
        "event_type": "webhook.received",
        "occurred_at": now,
        "source": {
            "domain": "events",
            "provider": "events.webhook",
            "handler_id": parsed.handler_id.clone().unwrap_or_else(|| "default".to_string()),
        },
        "scope": {
            "tenant": parsed.tenant.clone().unwrap_or_else(|| "default".to_string()),
            "team": parsed.team,
            "correlation_id": parsed.correlation_id,
        },
        "payload": parsed.event,
    });
    if let Some(http) = &parsed.http {
        emitted_event["http"] = http.clone();
    }
    if let Some(raw) = &parsed.raw {
        emitted_event["raw"] = raw.clone();
    }

    json_bytes(&json!({
        "ok": true,
        "receipt_id": receipt_id,
        "status": "accepted",
        "dispatched": false,
        "response": {
            "status": 200,
            "headers": { "content-type": "application/json" },
            "body": "accepted"
        },
        "emitted_events": [emitted_event],
    }))
}

fn handle_publish(input_json: &[u8]) -> Vec<u8> {
    let parsed: IngestInput = match serde_json::from_slice(input_json) {
        Ok(v) => v,
        Err(e) => {
            return json_bytes(&json!({"ok": false, "error": format!("invalid input: {e}")}));
        }
    };

    let Some(target_url) = parsed.config.target_url.as_deref() else {
        return json_bytes(&json!({
            "ok": false,
            "error": "target_url is required for publish"
        }));
    };

    let receipt_id = stable_receipt_id(&parsed.event);
    let request = build_request(&parsed.config, target_url, &parsed.event);
    let dispatched = dispatch_http(&request).is_ok();

    json_bytes(&json!({
        "ok": true,
        "receipt_id": receipt_id,
        "status": if dispatched { "published" } else { "queued" },
        "dispatched": dispatched,
        "request": request,
        "response": {
            "status": if dispatched { 200 } else { 202 },
            "headers": { "content-type": "application/json" },
            "body": "accepted"
        }
    }))
}

#[derive(Debug, Clone, Serialize)]
struct OutgoingRequest {
    method: String,
    url: String,
    headers: BTreeMap<String, String>,
    body: Value,
}

fn build_request(config: &IngestConfig, target_url: &str, event: &Value) -> OutgoingRequest {
    let mut headers = config.headers.clone();
    headers
        .entry("content-type".into())
        .or_insert_with(|| "application/json".into());
    if let Some(token) = &config.auth {
        headers
            .entry("authorization".into())
            .or_insert_with(|| format!("Bearer {token}"));
    }

    OutgoingRequest {
        method: config.method.to_uppercase(),
        url: target_url.to_string(),
        headers,
        body: json!({ "event": event }),
    }
}

fn stable_receipt_id(event: &Value) -> String {
    let bytes = serde_json::to_vec(event).unwrap_or_default();
    Uuid::new_v5(&Uuid::NAMESPACE_OID, &bytes).to_string()
}

fn dispatch_http(_request: &OutgoingRequest) -> Result<(), String> {
    // In WASM, we would use the http-client import here.
    // For now, we'll just return success for the demo.
    #[cfg(target_arch = "wasm32")]
    {
        use bindings::greentic::http::http_client;
        let headers: Vec<(String, String)> = _request
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let body = serde_json::to_vec(&_request.body).ok();
        let req = http_client::Request {
            method: _request.method.clone(),
            url: _request.url.clone(),
            headers,
            body,
        };
        match http_client::send(&req, None, None) {
            Ok(resp) if resp.status >= 200 && resp.status < 400 => Ok(()),
            Ok(resp) => Err(format!("http send returned {}", resp.status)),
            Err(e) => Err(format!("http send failed: {}", e.message)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Host tests run without HTTP; signal as queued/no-op.
        Err("http client not available on host".to_string())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn describe_produces_valid_payload() {
        let payload = build_describe_payload();
        assert_eq!(payload.provider, PROVIDER_ID);
        assert!(!payload.operations.is_empty());
        assert!(!payload.schema_hash.is_empty());
    }

    #[test]
    fn i18n_keys_cover_qa_specs() {
        use bindings::exports::greentic::component::qa::Mode;

        let keyset = I18N_KEYS
            .iter()
            .map(|value| (*value).to_string())
            .collect::<BTreeSet<_>>();

        for mode in [Mode::Default, Mode::Setup, Mode::Upgrade, Mode::Remove] {
            let spec = build_qa_spec(mode);
            assert!(keyset.contains(&spec.title.key));
            for question in spec.questions {
                assert!(keyset.contains(&question.label.key));
            }
        }
    }

    #[test]
    fn qa_default_asks_required_minimum() {
        use bindings::exports::greentic::component::qa::Mode;
        let spec = build_qa_spec(Mode::Default);
        let keys = spec
            .questions
            .into_iter()
            .map(|question| question.id)
            .collect::<Vec<_>>();
        assert!(keys.is_empty());
    }

    #[test]
    fn apply_answers_remove_returns_cleanup_plan() {
        use bindings::exports::greentic::component::qa::Guest as QaGuest;
        use bindings::exports::greentic::component::qa::Mode;
        let out =
            <Component as QaGuest>::apply_answers(Mode::Remove, canonical_cbor_bytes(&json!({})));
        let out_json: Value = decode_cbor(&out).expect("decode apply output");
        assert_eq!(out_json.get("ok"), Some(&Value::Bool(true)));
        assert_eq!(out_json.get("config"), Some(&Value::Null));
        let cleanup = out_json
            .get("remove")
            .and_then(|value| value.get("cleanup"))
            .and_then(Value::as_array)
            .expect("cleanup steps");
        assert!(!cleanup.is_empty());
    }

    #[test]
    fn apply_answers_validates_optional_target_url_when_present() {
        use bindings::exports::greentic::component::qa::Guest as QaGuest;
        use bindings::exports::greentic::component::qa::Mode;
        let answers = json!({
            "target_url": "not-a-url"
        });
        let out =
            <Component as QaGuest>::apply_answers(Mode::Default, canonical_cbor_bytes(&answers));
        let out_json: Value = decode_cbor(&out).expect("decode apply output");
        assert_eq!(out_json.get("ok"), Some(&Value::Bool(false)));
        let error = out_json
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert!(error.contains("URL"));
    }

    #[test]
    fn apply_answers_setup_produces_config() {
        use bindings::exports::greentic::component::qa::Guest as QaGuest;
        use bindings::exports::greentic::component::qa::Mode;
        let answers = json!({
            "enabled": true,
            "target_url": "https://example.com/webhook"
        });
        let out =
            <Component as QaGuest>::apply_answers(Mode::Setup, canonical_cbor_bytes(&answers));
        let out_json: Value = decode_cbor(&out).expect("decode apply output");
        assert_eq!(out_json.get("ok"), Some(&Value::Bool(true)));
        let config = out_json.get("config").expect("config object");
        assert_eq!(
            config.get("target_url"),
            Some(&Value::String("https://example.com/webhook".to_string()))
        );
        assert_eq!(
            config.get("method"),
            Some(&Value::String("POST".to_string()))
        );
    }

    #[test]
    fn apply_answers_setup_allows_inbound_only_config() {
        use bindings::exports::greentic::component::qa::Guest as QaGuest;
        use bindings::exports::greentic::component::qa::Mode;
        let answers = json!({
            "enabled": true
        });
        let out =
            <Component as QaGuest>::apply_answers(Mode::Setup, canonical_cbor_bytes(&answers));
        let out_json: Value = decode_cbor(&out).expect("decode apply output");
        assert_eq!(out_json.get("ok"), Some(&Value::Bool(true)));
        let config = out_json.get("config").expect("config object");
        assert_eq!(config.get("target_url"), Some(&Value::Null));
    }

    #[test]
    fn handle_ingest_returns_receipt_without_target_url() {
        let input = json!({
            "config": {
                "method": "POST"
            },
            "event": {"id": 1, "kind": "test"},
            "tenant": "test-tenant"
        });
        let result = handle_ingest_http(&serde_json::to_vec(&input).unwrap());
        let out: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(out.get("ok"), Some(&Value::Bool(true)));
        assert!(out.get("receipt_id").is_some());
        assert_eq!(
            out.get("status"),
            Some(&Value::String("accepted".to_string()))
        );
    }

    #[test]
    fn handle_publish_requires_target_url() {
        let input = json!({
            "config": {
                "method": "POST"
            },
            "event": {"id": 1, "kind": "test"},
        });
        let result = handle_publish(&serde_json::to_vec(&input).unwrap());
        let out: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(out.get("ok"), Some(&Value::Bool(false)));
        assert!(
            out.get("error")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .contains("target_url is required")
        );
    }

    #[test]
    fn handle_publish_returns_receipt() {
        let input = json!({
            "config": {
                "target_url": "https://example.com/hook",
                "method": "POST"
            },
            "event": {"id": 1, "kind": "test"},
            "tenant": "test-tenant"
        });
        let result = handle_publish(&serde_json::to_vec(&input).unwrap());
        let out: Value = serde_json::from_slice(&result).unwrap();
        assert_eq!(out.get("ok"), Some(&Value::Bool(true)));
        assert!(out.get("receipt_id").is_some());
    }

    #[test]
    fn receipt_is_deterministic() {
        let event = json!({"id": 1, "kind": "test"});
        let id1 = stable_receipt_id(&event);
        let id2 = stable_receipt_id(&event);
        assert_eq!(id1, id2);
    }
}
