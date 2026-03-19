//! Shared utility functions for event provider components.

use crate::component_v0_6::{
    AdaptiveCardInputMode, DescribePayload, I18nText, OperationDescriptor, QaQuestionSpec, QaSpec,
    QuestionKind, SchemaField, SchemaIr, canonical_cbor_bytes, decode_cbor,
    default_en_i18n_messages,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// JSON serialization
// ---------------------------------------------------------------------------

/// Serialize a value to JSON bytes, returning `{}` on failure.
pub fn json_bytes<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap_or_else(|_| b"{}".to_vec())
}

// ---------------------------------------------------------------------------
// I18n / descriptor helpers
// ---------------------------------------------------------------------------

/// Build an [`I18nText`] from a dotted key.
pub fn i18n(key: &str) -> I18nText {
    I18nText {
        key: key.to_string(),
    }
}

/// Build an [`OperationDescriptor`] from a name and i18n keys.
pub fn op(name: &str, title_key: &str, desc_key: &str) -> OperationDescriptor {
    OperationDescriptor {
        name: name.to_string(),
        title: i18n(title_key),
        description: i18n(desc_key),
    }
}

/// Build a [`QaQuestionSpec`] from a key, i18n text key, and required flag.
pub fn qa_q(key: &str, text_key: &str, required: bool) -> QaQuestionSpec {
    QaQuestionSpec {
        id: key.to_string(),
        label: i18n(text_key),
        help: None,
        error: None,
        kind: QuestionKind::Text,
        required,
        default: None,
        skip_if: None,
    }
}

/// Build a [`QaQuestionSpec`] for boolean input.
pub fn qa_bool(key: &str, text_key: &str, required: bool) -> QaQuestionSpec {
    QaQuestionSpec {
        id: key.to_string(),
        label: i18n(text_key),
        help: None,
        error: None,
        kind: QuestionKind::Bool,
        required,
        default: None,
        skip_if: None,
    }
}

/// Build a [`QaQuestionSpec`] for number input.
pub fn qa_number(key: &str, text_key: &str, required: bool) -> QaQuestionSpec {
    QaQuestionSpec {
        id: key.to_string(),
        label: i18n(text_key),
        help: None,
        error: None,
        kind: QuestionKind::Number,
        required,
        default: None,
        skip_if: None,
    }
}

/// Build a [`QaQuestionSpec`] for inline adaptive card JSON input.
pub fn qa_adaptive_card_inline(key: &str, text_key: &str, required: bool) -> QaQuestionSpec {
    QaQuestionSpec {
        id: key.to_string(),
        label: i18n(text_key),
        help: None,
        error: None,
        kind: QuestionKind::AdaptiveCard {
            input_mode: AdaptiveCardInputMode::Inline,
            schema: None,
            file_types: vec!["json".to_string()],
            base_path: None,
            check_exists: true,
        },
        required,
        default: None,
        skip_if: None,
    }
}

/// Build a [`QaQuestionSpec`] for adaptive card file path input.
pub fn qa_adaptive_card_file(
    key: &str,
    text_key: &str,
    required: bool,
    base_path: Option<&str>,
) -> QaQuestionSpec {
    QaQuestionSpec {
        id: key.to_string(),
        label: i18n(text_key),
        help: None,
        error: None,
        kind: QuestionKind::AdaptiveCard {
            input_mode: AdaptiveCardInputMode::File,
            schema: None,
            file_types: vec!["json".to_string()],
            base_path: base_path.map(|s| s.to_string()),
            check_exists: true,
        },
        required,
        default: None,
        skip_if: None,
    }
}

/// Build a [`QaQuestionSpec`] for adaptive card with custom options.
pub fn qa_adaptive_card(
    key: &str,
    text_key: &str,
    required: bool,
    input_mode: AdaptiveCardInputMode,
    base_path: Option<&str>,
    schema: Option<serde_json::Value>,
) -> QaQuestionSpec {
    QaQuestionSpec {
        id: key.to_string(),
        label: i18n(text_key),
        help: None,
        error: None,
        kind: QuestionKind::AdaptiveCard {
            input_mode,
            schema,
            file_types: vec!["json".to_string()],
            base_path: base_path.map(|s| s.to_string()),
            check_exists: true,
        },
        required,
        default: None,
        skip_if: None,
    }
}

// ---------------------------------------------------------------------------
// QA answer extraction
// ---------------------------------------------------------------------------

/// Extract a string from `answers[key]`, falling back to `default`.
pub fn string_or_default(answers: &Value, key: &str, default: &str) -> String {
    answers
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
        .to_string()
}

/// Extract an optional non-empty string from `answers[key]`.
pub fn optional_string_from(answers: &Value, key: &str) -> Option<String> {
    let value = answers.get(key)?;
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        _ => None,
    }
}

/// Deserialize `answers.existing_config` or `answers.config` into `T`.
pub fn existing_config_from_answers<T: DeserializeOwned>(answers: &Value) -> Option<T> {
    answers
        .get("existing_config")
        .cloned()
        .or_else(|| answers.get("config").cloned())
        .and_then(|value| serde_json::from_value(value).ok())
}

// ---------------------------------------------------------------------------
// I18n boilerplate helpers
// ---------------------------------------------------------------------------

/// Convert a `&[&str]` key list to `Vec<String>` for `i18n_keys()`.
pub fn i18n_keys_from(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|k| (*k).to_string()).collect()
}

/// Build a default English i18n bundle CBOR blob for `i18n_bundle()`.
pub fn i18n_bundle_default(locale: String, keys: &[&str]) -> Vec<u8> {
    let locale = if locale.trim().is_empty() {
        "en".to_string()
    } else {
        locale
    };
    let messages = default_en_i18n_messages(keys);
    canonical_cbor_bytes(&json!({"locale": locale, "messages": Value::Object(messages)}))
}

/// Build an i18n bundle CBOR blob from explicit `(key, value)` pairs.
pub fn i18n_bundle_from_pairs(locale: String, pairs: &[(&str, &str)]) -> Vec<u8> {
    let locale = if locale.trim().is_empty() {
        "en".to_string()
    } else {
        locale
    };
    let messages: serde_json::Map<String, Value> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), Value::String((*v).to_string())))
        .collect();
    canonical_cbor_bytes(&json!({"locale": locale, "messages": Value::Object(messages)}))
}

// ---------------------------------------------------------------------------
// Schema builder helpers
// ---------------------------------------------------------------------------

/// Build a `SchemaIr::String` (non-secret, no format).
pub fn schema_str(title: &str, desc: &str) -> SchemaIr {
    SchemaIr::String {
        title: i18n(title),
        description: i18n(desc),
        format: None,
        secret: false,
    }
}

/// Build a `SchemaIr::String` with a format (e.g. `"uri"`).
pub fn schema_str_fmt(title: &str, desc: &str, format: &str) -> SchemaIr {
    SchemaIr::String {
        title: i18n(title),
        description: i18n(desc),
        format: Some(format.to_string()),
        secret: false,
    }
}

/// Build a secret `SchemaIr::String` (no format).
pub fn schema_secret(title: &str, desc: &str) -> SchemaIr {
    SchemaIr::String {
        title: i18n(title),
        description: i18n(desc),
        format: None,
        secret: true,
    }
}

/// Build a `SchemaIr::Bool`.
pub fn schema_bool_ir(title: &str, desc: &str) -> SchemaIr {
    SchemaIr::Bool {
        title: i18n(title),
        description: i18n(desc),
    }
}

/// Build a `SchemaIr::Number`.
pub fn schema_number_ir(title: &str, desc: &str) -> SchemaIr {
    SchemaIr::Number {
        title: i18n(title),
        description: i18n(desc),
    }
}

/// Build a `SchemaIr::Object` from a list of `(name, required, schema)`.
pub fn schema_obj(
    title: &str,
    desc: &str,
    field_defs: Vec<(&str, bool, SchemaIr)>,
    additional_properties: bool,
) -> SchemaIr {
    let mut fields = BTreeMap::new();
    for (name, required, schema) in field_defs {
        fields.insert(name.to_string(), SchemaField { required, schema });
    }
    SchemaIr::Object {
        title: i18n(title),
        description: i18n(desc),
        fields,
        additional_properties,
    }
}

// ---------------------------------------------------------------------------
// CBOR-JSON invoke bridge
// ---------------------------------------------------------------------------

/// Decode CBOR input, dispatch to `dispatch_fn`, and re-encode the JSON result as CBOR.
pub fn cbor_json_invoke_bridge(
    op: &str,
    input_cbor: &[u8],
    run_alias: Option<&str>,
    dispatch_fn: impl FnOnce(&str, &[u8]) -> Vec<u8>,
) -> Vec<u8> {
    let input_value: Value = match decode_cbor(input_cbor) {
        Ok(value) => value,
        Err(err) => {
            return canonical_cbor_bytes(
                &json!({"ok": false, "error": format!("invalid input cbor: {err}")}),
            );
        }
    };
    let input_json = serde_json::to_vec(&input_value).unwrap_or_default();
    let effective_op = if op == "run" {
        run_alias.unwrap_or(op)
    } else {
        op
    };
    let output_json = dispatch_fn(effective_op, &input_json);
    let output_value: Value = serde_json::from_slice(&output_json)
        .unwrap_or_else(|_| json!({"ok": false, "error": "provider produced invalid json"}));
    canonical_cbor_bytes(&output_value)
}

/// schema-core-api `describe()` — JSON-serialize a [`DescribePayload`].
pub fn schema_core_describe(payload: &DescribePayload) -> Vec<u8> {
    serde_json::to_vec(payload).unwrap_or_default()
}

/// schema-core-api `validate_config()` — always returns `{"ok": true}`.
pub fn schema_core_validate_config() -> Vec<u8> {
    json_bytes(&json!({"ok": true}))
}

/// schema-core-api `healthcheck()` — always returns `{"status": "healthy"}`.
pub fn schema_core_healthcheck() -> Vec<u8> {
    json_bytes(&json!({"status": "healthy"}))
}

// ---------------------------------------------------------------------------
// QA spec builder
// ---------------------------------------------------------------------------

/// Question definition: `(key, i18n_text_key, required_in_setup)`.
pub type QaQuestionDef<'a> = (&'a str, &'a str, bool);

/// Build a [`QaSpec`] for the given mode from shared question definitions.
///
/// - `prefix`: provider prefix (e.g. `"webhook"`)
/// - `setup_questions`: full list of `(key, text_key, required)` for Setup
/// - `default_keys`: subset of keys that appear in Default mode (all required)
///
/// Upgrade mode reuses Setup questions with `required = false`.
/// Remove mode returns an empty question list.
pub fn qa_spec_for_mode(
    mode: &str,
    prefix: &str,
    setup_questions: &[QaQuestionDef],
    default_keys: &[&str],
) -> QaSpec {
    match mode {
        "default" => {
            let questions = default_keys
                .iter()
                .filter_map(|dk| {
                    setup_questions
                        .iter()
                        .find(|(k, _, _)| k == dk)
                        .map(|(k, t, _)| qa_q(k, t, true))
                })
                .collect();
            QaSpec {
                mode: "default".to_string(),
                title: i18n(&format!("{prefix}.qa.default.title")),
                description: None,
                questions,
                defaults: Default::default(),
            }
        }
        "setup" => QaSpec {
            mode: "setup".to_string(),
            title: i18n(&format!("{prefix}.qa.setup.title")),
            description: None,
            questions: setup_questions
                .iter()
                .map(|(k, t, r)| qa_q(k, t, *r))
                .collect(),
            defaults: Default::default(),
        },
        "upgrade" => QaSpec {
            mode: "upgrade".to_string(),
            title: i18n(&format!("{prefix}.qa.upgrade.title")),
            description: None,
            questions: setup_questions
                .iter()
                .map(|(k, t, _)| qa_q(k, t, false))
                .collect(),
            defaults: Default::default(),
        },
        _ => QaSpec {
            mode: "remove".to_string(),
            title: i18n(&format!("{prefix}.qa.remove.title")),
            description: None,
            questions: Vec::new(),
            defaults: Default::default(),
        },
    }
}

// ---------------------------------------------------------------------------
// Config loader
// ---------------------------------------------------------------------------

/// Load a provider config from input JSON.
pub fn load_config_generic<T: DeserializeOwned>(input: &Value, keys: &[&str]) -> Result<T, String> {
    if let Some(cfg) = input.get("config") {
        return serde_json::from_value::<T>(cfg.clone())
            .map_err(|e| format!("invalid config: {e}"));
    }
    let mut partial = serde_json::Map::new();
    for key in keys {
        if let Some(v) = input.get(*key) {
            partial.insert((*key).to_string(), v.clone());
        }
    }
    if !partial.is_empty() {
        return serde_json::from_value::<T>(Value::Object(partial))
            .map_err(|e| format!("invalid config: {e}"));
    }
    Err("missing config: expected `config` or top-level config fields".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_bytes_produces_valid_json() {
        let bytes = json_bytes(&json!({"ok": true}));
        let parsed: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["ok"], true);
    }

    #[test]
    fn string_or_default_returns_value_when_present() {
        let answers = json!({"name": "Alice"});
        assert_eq!(string_or_default(&answers, "name", "Bob"), "Alice");
    }

    #[test]
    fn string_or_default_returns_default_when_missing() {
        let answers = json!({});
        assert_eq!(string_or_default(&answers, "name", "Bob"), "Bob");
    }

    #[test]
    fn optional_string_from_returns_some() {
        let answers = json!({"name": "Alice"});
        assert_eq!(optional_string_from(&answers, "name"), Some("Alice".into()));
    }

    #[test]
    fn optional_string_from_returns_none_for_empty() {
        let answers = json!({"name": ""});
        assert_eq!(optional_string_from(&answers, "name"), None);
    }

    #[test]
    fn qa_spec_for_mode_builds_setup() {
        let questions: &[QaQuestionDef] = &[
            ("target_url", "webhook.qa.target_url", true),
            ("timeout", "webhook.qa.timeout", false),
        ];
        let spec = qa_spec_for_mode("setup", "webhook", questions, &["target_url"]);
        assert_eq!(spec.mode, "setup");
        assert_eq!(spec.questions.len(), 2);
    }

    #[test]
    fn qa_spec_for_mode_builds_remove() {
        let questions: &[QaQuestionDef] = &[("target_url", "webhook.qa.target_url", true)];
        let spec = qa_spec_for_mode("remove", "webhook", questions, &["target_url"]);
        assert_eq!(spec.mode, "remove");
        assert!(spec.questions.is_empty());
    }

    #[test]
    fn op_builds_descriptor() {
        let desc = op("send", "p.send.title", "p.send.desc");
        assert_eq!(desc.name, "send");
        assert_eq!(desc.title.key, "p.send.title");
        assert_eq!(desc.description.key, "p.send.desc");
    }

    #[test]
    fn schema_core_validate_config_returns_ok() {
        let bytes = schema_core_validate_config();
        let val: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(val["ok"], true);
    }

    #[test]
    fn schema_core_healthcheck_returns_healthy() {
        let bytes = schema_core_healthcheck();
        let val: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(val["status"], "healthy");
    }
}
