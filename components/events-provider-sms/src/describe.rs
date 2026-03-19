//! Provider description and QA specs for SMS event provider.

use provider_common::component_v0_6::{
    DescribePayload, QaSpec, SchemaIr, canonical_cbor_bytes, schema_hash,
};
use provider_common::helpers::{op, schema_bool_ir, schema_obj, schema_str};
use serde_json::{Value, json};

use crate::{PROVIDER_ID, WORLD_ID};

/// I18n keys for the SMS provider.
pub(crate) const I18N_KEYS: &[&str] = &[
    "sms.op.ingest_http.title",
    "sms.op.ingest_http.description",
    "sms.op.publish.title",
    "sms.op.publish.description",
    "sms.schema.input.title",
    "sms.schema.input.description",
    "sms.schema.input.event.title",
    "sms.schema.input.event.description",
    "sms.schema.output.title",
    "sms.schema.output.description",
    "sms.schema.output.ok.title",
    "sms.schema.output.ok.description",
    "sms.schema.output.receipt_id.title",
    "sms.schema.output.receipt_id.description",
    "sms.schema.config.title",
    "sms.schema.config.description",
    "sms.schema.config.enabled.title",
    "sms.schema.config.enabled.description",
    "sms.schema.config.messaging_provider_id.title",
    "sms.schema.config.messaging_provider_id.description",
    "sms.schema.config.from.title",
    "sms.schema.config.from.description",
    "sms.schema.config.persistence_key_prefix.title",
    "sms.schema.config.persistence_key_prefix.description",
    "sms.qa.default.title",
    "sms.qa.setup.title",
    "sms.qa.upgrade.title",
    "sms.qa.remove.title",
    "sms.qa.setup.enabled",
    "sms.qa.setup.messaging_provider_id",
    "sms.qa.setup.from",
    "sms.qa.setup.persistence_key_prefix",
];

/// Setup questions for the SMS provider.
pub(crate) const SETUP_QUESTIONS: &[provider_common::helpers::QaQuestionDef] = &[
    ("enabled", "sms.qa.setup.enabled", true),
    (
        "messaging_provider_id",
        "sms.qa.setup.messaging_provider_id",
        true,
    ),
    ("from", "sms.qa.setup.from", false),
    (
        "persistence_key_prefix",
        "sms.qa.setup.persistence_key_prefix",
        false,
    ),
];

/// Keys for default mode (minimal required config).
pub(crate) const DEFAULT_KEYS: &[&str] = &["messaging_provider_id"];

/// I18n pairs for English locale.
pub(crate) const I18N_PAIRS: &[(&str, &str)] = &[
    ("sms.op.ingest_http.title", "Ingest HTTP"),
    (
        "sms.op.ingest_http.description",
        "Process incoming SMS event",
    ),
    ("sms.op.publish.title", "Publish"),
    ("sms.op.publish.description", "Publish SMS event"),
    ("sms.schema.input.title", "SMS input"),
    ("sms.schema.input.description", "Input for SMS operations"),
    ("sms.schema.input.event.title", "Event"),
    ("sms.schema.input.event.description", "Event payload"),
    ("sms.schema.output.title", "SMS output"),
    ("sms.schema.output.description", "Result of SMS operation"),
    ("sms.schema.output.ok.title", "Success"),
    (
        "sms.schema.output.ok.description",
        "Whether operation succeeded",
    ),
    ("sms.schema.output.receipt_id.title", "Receipt ID"),
    (
        "sms.schema.output.receipt_id.description",
        "Unique identifier for the processed event",
    ),
    ("sms.schema.config.title", "SMS config"),
    (
        "sms.schema.config.description",
        "SMS provider configuration",
    ),
    ("sms.schema.config.enabled.title", "Enabled"),
    (
        "sms.schema.config.enabled.description",
        "Enable this provider",
    ),
    (
        "sms.schema.config.messaging_provider_id.title",
        "Messaging Provider ID",
    ),
    (
        "sms.schema.config.messaging_provider_id.description",
        "ID of the messaging provider for SMS delivery",
    ),
    ("sms.schema.config.from.title", "From Number"),
    (
        "sms.schema.config.from.description",
        "Default sender phone number",
    ),
    (
        "sms.schema.config.persistence_key_prefix.title",
        "Persistence Key Prefix",
    ),
    (
        "sms.schema.config.persistence_key_prefix.description",
        "Prefix for state store keys",
    ),
    ("sms.qa.default.title", "Default"),
    ("sms.qa.setup.title", "Setup"),
    ("sms.qa.upgrade.title", "Upgrade"),
    ("sms.qa.remove.title", "Remove"),
    ("sms.qa.setup.enabled", "Enable provider"),
    (
        "sms.qa.setup.messaging_provider_id",
        "Messaging Provider ID",
    ),
    ("sms.qa.setup.from", "From Number"),
    (
        "sms.qa.setup.persistence_key_prefix",
        "Persistence Key Prefix",
    ),
];

/// Build the describe payload for the SMS provider.
pub(crate) fn build_describe_payload() -> DescribePayload {
    let input_schema = input_schema();
    let output_schema = output_schema();
    let config_schema = config_schema();
    DescribePayload {
        provider: PROVIDER_ID.to_string(),
        world: WORLD_ID.to_string(),
        operations: vec![
            op(
                "ingest_http",
                "sms.op.ingest_http.title",
                "sms.op.ingest_http.description",
            ),
            op(
                "publish",
                "sms.op.publish.title",
                "sms.op.publish.description",
            ),
        ],
        input_schema: input_schema.clone(),
        output_schema: output_schema.clone(),
        config_schema: config_schema.clone(),
        redactions: vec![],
        schema_hash: schema_hash(&input_schema, &output_schema, &config_schema),
    }
}

/// Build a QA spec for the given mode.
pub(crate) fn build_qa_spec(
    mode: crate::bindings::exports::greentic::component::qa::Mode,
) -> QaSpec {
    use crate::bindings::exports::greentic::component::qa::Mode;
    let mode_str = match mode {
        Mode::Default => "default",
        Mode::Setup => "setup",
        Mode::Upgrade => "upgrade",
        Mode::Remove => "remove",
    };
    provider_common::helpers::qa_spec_for_mode(mode_str, "sms", SETUP_QUESTIONS, DEFAULT_KEYS)
}

fn input_schema() -> SchemaIr {
    schema_obj(
        "sms.schema.input.title",
        "sms.schema.input.description",
        vec![(
            "event",
            true,
            schema_str(
                "sms.schema.input.event.title",
                "sms.schema.input.event.description",
            ),
        )],
        true,
    )
}

fn output_schema() -> SchemaIr {
    schema_obj(
        "sms.schema.output.title",
        "sms.schema.output.description",
        vec![
            (
                "ok",
                true,
                schema_bool_ir(
                    "sms.schema.output.ok.title",
                    "sms.schema.output.ok.description",
                ),
            ),
            (
                "receipt_id",
                false,
                schema_str(
                    "sms.schema.output.receipt_id.title",
                    "sms.schema.output.receipt_id.description",
                ),
            ),
        ],
        true,
    )
}

fn config_schema() -> SchemaIr {
    schema_obj(
        "sms.schema.config.title",
        "sms.schema.config.description",
        vec![
            (
                "enabled",
                true,
                schema_bool_ir(
                    "sms.schema.config.enabled.title",
                    "sms.schema.config.enabled.description",
                ),
            ),
            (
                "messaging_provider_id",
                true,
                schema_str(
                    "sms.schema.config.messaging_provider_id.title",
                    "sms.schema.config.messaging_provider_id.description",
                ),
            ),
            (
                "from",
                false,
                schema_str(
                    "sms.schema.config.from.title",
                    "sms.schema.config.from.description",
                ),
            ),
            (
                "persistence_key_prefix",
                false,
                schema_str(
                    "sms.schema.config.persistence_key_prefix.title",
                    "sms.schema.config.persistence_key_prefix.description",
                ),
            ),
        ],
        false,
    )
}

/// Build an i18n bundle for the given locale.
pub(crate) fn i18n_bundle(locale: String) -> Vec<u8> {
    let locale = if locale.trim().is_empty() {
        "en".to_string()
    } else {
        locale
    };
    let messages: serde_json::Map<String, Value> = I18N_PAIRS
        .iter()
        .map(|(key, value)| ((*key).to_string(), Value::String((*value).to_string())))
        .collect();
    canonical_cbor_bytes(&json!({"locale": locale, "messages": Value::Object(messages)}))
}
