//! QASpec types and CBOR utilities for component v0.6.0.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Operation descriptor for provider describe payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperationDescriptor {
    pub name: String,
    pub title: I18nText,
    pub description: I18nText,
}

/// Redaction rule for sensitive data in logs/traces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedactionRule {
    pub path: String,
    pub strategy: String,
}

/// Provider describe payload (v0.6.0 format).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DescribePayload {
    pub provider: String,
    pub world: String,
    pub operations: Vec<OperationDescriptor>,
    pub input_schema: SchemaIr,
    pub output_schema: SchemaIr,
    pub config_schema: SchemaIr,
    pub redactions: Vec<RedactionRule>,
    pub schema_hash: String,
}

/// Intermediate schema representation for input/output/config schemas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SchemaIr {
    Bool {
        title: I18nText,
        description: I18nText,
    },
    String {
        title: I18nText,
        description: I18nText,
        format: Option<String>,
        secret: bool,
    },
    Number {
        title: I18nText,
        description: I18nText,
    },
    Object {
        title: I18nText,
        description: I18nText,
        fields: BTreeMap<String, SchemaField>,
        additional_properties: bool,
    },
}

/// Schema field with required flag and nested schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaField {
    pub required: bool,
    pub schema: SchemaIr,
}

/// I18n text reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct I18nText {
    pub key: String,
}

/// Input mode for adaptive card questions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AdaptiveCardInputMode {
    /// Inline JSON input via text area (copy-paste).
    #[default]
    Inline,
    /// File path pointing to an adaptive card JSON asset.
    File,
}

/// Question kind — matches `ComponentQaSpec.QuestionKind` in greentic-types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum QuestionKind {
    Text,
    Choice {
        options: Vec<ChoiceOption>,
    },
    Number,
    Bool,
    InlineJson {
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
    },
    AssetRef {
        #[serde(default)]
        file_types: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        base_path: Option<String>,
        #[serde(default = "default_true")]
        check_exists: bool,
    },
    /// Adaptive Card input — supports inline JSON or file path reference.
    AdaptiveCard {
        /// Input mode: inline JSON text area or file path reference.
        #[serde(default)]
        input_mode: AdaptiveCardInputMode,
        /// Optional JSON Schema for validation (Adaptive Card schema).
        #[serde(skip_serializing_if = "Option::is_none")]
        schema: Option<serde_json::Value>,
        /// Allowed file extensions for file mode (defaults to ["json"]).
        #[serde(default = "default_adaptive_card_file_types")]
        file_types: Vec<String>,
        /// Base path for resolving file references.
        #[serde(skip_serializing_if = "Option::is_none")]
        base_path: Option<String>,
        /// Whether to verify file exists (file mode only).
        #[serde(default = "default_true")]
        check_exists: bool,
    },
}

fn default_true() -> bool {
    true
}

fn default_adaptive_card_file_types() -> Vec<String> {
    vec!["json".to_string()]
}

/// Choice option for `QuestionKind::Choice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChoiceOption {
    pub value: String,
    pub label: I18nText,
}

/// Skip condition expression — supports AND/OR with nesting.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SkipExpression {
    Condition(SkipCondition),
    And(Vec<SkipExpression>),
    Or(Vec<SkipExpression>),
    Not(Box<SkipExpression>),
}

/// Single skip condition for field comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkipCondition {
    pub field: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equals: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_equals: Option<serde_json::Value>,
    #[serde(default)]
    pub is_empty: bool,
    #[serde(default)]
    pub is_not_empty: bool,
}

/// QA question — matches `ComponentQaSpec.Question` in greentic-types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QaQuestionSpec {
    pub id: String,
    pub label: I18nText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<I18nText>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<I18nText>,
    pub kind: QuestionKind,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_if: Option<SkipExpression>,
}

/// QA spec — matches `ComponentQaSpec` in greentic-types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QaSpec {
    pub mode: String,
    pub title: I18nText,
    #[serde(default)]
    pub description: Option<I18nText>,
    pub questions: Vec<QaQuestionSpec>,
    #[serde(default)]
    pub defaults: BTreeMap<String, serde_json::Value>,
}

/// Compute schema hash from input, output, and config schemas.
pub fn schema_hash(input: &SchemaIr, output: &SchemaIr, config: &SchemaIr) -> String {
    let value = serde_json::json!({
        "input": input,
        "output": output,
        "config": config,
    });
    sha256_hex(&to_canonical_cbor(&value))
}

/// Serialize a value to canonical CBOR bytes.
pub fn canonical_cbor_bytes(value: &impl Serialize) -> Vec<u8> {
    to_canonical_cbor(value)
}

/// Serialize a value to canonical CBOR bytes.
pub fn to_canonical_cbor(value: &impl Serialize) -> Vec<u8> {
    let value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    let canonical = canonicalize_json(value);
    let mut out = Vec::new();
    let _ = ciborium::ser::into_writer(&canonical, &mut out);
    out
}

/// Decode CBOR bytes to a typed value.
pub fn decode_cbor<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, String> {
    ciborium::de::from_reader(bytes).map_err(|err| err.to_string())
}

/// Generate a default English message from an i18n key.
pub fn default_en_message_for_key(key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        return "Message".to_string();
    }

    let mut words = key
        .split('.')
        .next_back()
        .unwrap_or(key)
        .split('_')
        .filter_map(|token| {
            let trimmed = token.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_ascii_lowercase())
            }
        })
        .collect::<Vec<_>>();

    if words.is_empty() {
        return "Message".to_string();
    }

    for word in &mut words {
        match word.as_str() {
            "qa" | "op" | "schema" | "config" | "input" | "output" => {}
            "id" => *word = "ID".to_string(),
            "url" => *word = "URL".to_string(),
            "http" => *word = "HTTP".to_string(),
            "api" => *word = "API".to_string(),
            "ui" => *word = "UI".to_string(),
            "i18n" => *word = "I18N".to_string(),
            _ => {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    *word = format!("{}{}", first.to_ascii_uppercase(), chars.as_str());
                }
            }
        }
    }

    words.join(" ")
}

/// Build default English i18n messages from a list of keys.
pub fn default_en_i18n_messages(keys: &[&str]) -> serde_json::Map<String, serde_json::Value> {
    keys.iter()
        .map(|key| {
            (
                (*key).to_string(),
                serde_json::Value::String(default_en_message_for_key(key)),
            )
        })
        .collect()
}

/// Compute SHA256 hex digest of bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn canonicalize_json(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(canonicalize_json).collect())
        }
        serde_json::Value::Object(map) => {
            let mut sorted = BTreeMap::new();
            for (key, value) in map {
                sorted.insert(key, canonicalize_json(value));
            }
            let ordered = sorted
                .into_iter()
                .collect::<serde_json::Map<String, serde_json::Value>>();
            serde_json::Value::Object(ordered)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_human_readable_default_i18n_message() {
        assert_eq!(
            default_en_message_for_key("webhook.qa.setup.target_url"),
            "Target URL"
        );
        assert_eq!(
            default_en_message_for_key("timer.schema.output.event_id.title"),
            "Title"
        );
        assert_eq!(default_en_message_for_key(""), "Message");
    }

    #[test]
    fn cbor_roundtrip_works() {
        use serde_json::json;
        let value = json!({"key": "value", "number": 42});
        let cbor = canonical_cbor_bytes(&value);
        let decoded: serde_json::Value = decode_cbor(&cbor).unwrap();
        assert_eq!(decoded["key"], "value");
        assert_eq!(decoded["number"], 42);
    }

    #[test]
    fn schema_hash_is_deterministic() {
        let schema = SchemaIr::String {
            title: I18nText {
                key: "test.title".to_string(),
            },
            description: I18nText {
                key: "test.desc".to_string(),
            },
            format: None,
            secret: false,
        };
        let hash1 = schema_hash(&schema, &schema, &schema);
        let hash2 = schema_hash(&schema, &schema, &schema);
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
}
