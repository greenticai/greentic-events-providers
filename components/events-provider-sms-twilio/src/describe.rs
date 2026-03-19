//! Provider description and QA specs for Twilio SMS event provider.

use provider_common::component_v0_6::{
    DescribePayload, QaSpec, SchemaIr, canonical_cbor_bytes, schema_hash,
};
use provider_common::helpers::{op, schema_bool_ir, schema_obj, schema_str};
use serde_json::{Value, json};

use crate::{PROVIDER_ID, WORLD_ID};

/// I18n keys for the Twilio SMS provider.
pub(crate) const I18N_KEYS: &[&str] = &[
    "twilio.op.ingest_http.title",
    "twilio.op.ingest_http.description",
    "twilio.op.publish.title",
    "twilio.op.publish.description",
    "twilio.op.send_sms.title",
    "twilio.op.send_sms.description",
    "twilio.schema.input.title",
    "twilio.schema.input.description",
    "twilio.schema.input.event.title",
    "twilio.schema.input.event.description",
    "twilio.schema.output.title",
    "twilio.schema.output.description",
    "twilio.schema.output.ok.title",
    "twilio.schema.output.ok.description",
    "twilio.schema.output.receipt_id.title",
    "twilio.schema.output.receipt_id.description",
    "twilio.schema.config.title",
    "twilio.schema.config.description",
    "twilio.schema.config.enabled.title",
    "twilio.schema.config.enabled.description",
    "twilio.schema.config.messaging_provider_id.title",
    "twilio.schema.config.messaging_provider_id.description",
    "twilio.schema.config.from.title",
    "twilio.schema.config.from.description",
    "twilio.schema.config.persistence_key_prefix.title",
    "twilio.schema.config.persistence_key_prefix.description",
    "twilio.qa.default.title",
    "twilio.qa.setup.title",
    "twilio.qa.upgrade.title",
    "twilio.qa.remove.title",
    "twilio.qa.setup.enabled",
    "twilio.qa.setup.messaging_provider_id",
    "twilio.qa.setup.from",
    "twilio.qa.setup.persistence_key_prefix",
];

/// Setup questions for the Twilio SMS provider.
pub(crate) const SETUP_QUESTIONS: &[provider_common::helpers::QaQuestionDef] = &[
    ("enabled", "twilio.qa.setup.enabled", true),
    (
        "messaging_provider_id",
        "twilio.qa.setup.messaging_provider_id",
        true,
    ),
    ("from", "twilio.qa.setup.from", false),
    (
        "persistence_key_prefix",
        "twilio.qa.setup.persistence_key_prefix",
        false,
    ),
];

/// Keys for default mode (minimal required config).
pub(crate) const DEFAULT_KEYS: &[&str] = &["messaging_provider_id"];

/// I18n pairs for English locale.
pub(crate) const I18N_PAIRS: &[(&str, &str)] = &[
    ("twilio.op.ingest_http.title", "Ingest HTTP"),
    (
        "twilio.op.ingest_http.description",
        "Process incoming Twilio SMS event",
    ),
    ("twilio.op.publish.title", "Publish"),
    ("twilio.op.publish.description", "Publish Twilio SMS event"),
    ("twilio.op.send_sms.title", "Send SMS"),
    (
        "twilio.op.send_sms.description",
        "Send outbound SMS via Twilio",
    ),
    ("twilio.schema.input.title", "Twilio SMS input"),
    (
        "twilio.schema.input.description",
        "Input for Twilio SMS operations",
    ),
    ("twilio.schema.input.event.title", "Event"),
    ("twilio.schema.input.event.description", "Event payload"),
    ("twilio.schema.output.title", "Twilio SMS output"),
    (
        "twilio.schema.output.description",
        "Result of Twilio SMS operation",
    ),
    ("twilio.schema.output.ok.title", "Success"),
    (
        "twilio.schema.output.ok.description",
        "Whether operation succeeded",
    ),
    ("twilio.schema.output.receipt_id.title", "Receipt ID"),
    (
        "twilio.schema.output.receipt_id.description",
        "Unique identifier for the processed event",
    ),
    ("twilio.schema.config.title", "Twilio SMS config"),
    (
        "twilio.schema.config.description",
        "Twilio SMS provider configuration",
    ),
    ("twilio.schema.config.enabled.title", "Enabled"),
    (
        "twilio.schema.config.enabled.description",
        "Enable this provider",
    ),
    (
        "twilio.schema.config.messaging_provider_id.title",
        "Messaging Provider ID",
    ),
    (
        "twilio.schema.config.messaging_provider_id.description",
        "ID of the messaging provider for SMS delivery",
    ),
    ("twilio.schema.config.from.title", "From Number"),
    (
        "twilio.schema.config.from.description",
        "Default Twilio sender phone number",
    ),
    (
        "twilio.schema.config.persistence_key_prefix.title",
        "Persistence Key Prefix",
    ),
    (
        "twilio.schema.config.persistence_key_prefix.description",
        "Prefix for state store keys",
    ),
    ("twilio.qa.default.title", "Default"),
    ("twilio.qa.setup.title", "Setup"),
    ("twilio.qa.upgrade.title", "Upgrade"),
    ("twilio.qa.remove.title", "Remove"),
    ("twilio.qa.setup.enabled", "Enable provider"),
    (
        "twilio.qa.setup.messaging_provider_id",
        "Messaging Provider ID",
    ),
    ("twilio.qa.setup.from", "From Number"),
    (
        "twilio.qa.setup.persistence_key_prefix",
        "Persistence Key Prefix",
    ),
];

/// Build the describe payload for the Twilio SMS provider.
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
                "twilio.op.ingest_http.title",
                "twilio.op.ingest_http.description",
            ),
            op(
                "publish",
                "twilio.op.publish.title",
                "twilio.op.publish.description",
            ),
            op(
                "send_sms",
                "twilio.op.send_sms.title",
                "twilio.op.send_sms.description",
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
    provider_common::helpers::qa_spec_for_mode(mode_str, "twilio", SETUP_QUESTIONS, DEFAULT_KEYS)
}

fn input_schema() -> SchemaIr {
    schema_obj(
        "twilio.schema.input.title",
        "twilio.schema.input.description",
        vec![(
            "event",
            true,
            schema_str(
                "twilio.schema.input.event.title",
                "twilio.schema.input.event.description",
            ),
        )],
        true,
    )
}

fn output_schema() -> SchemaIr {
    schema_obj(
        "twilio.schema.output.title",
        "twilio.schema.output.description",
        vec![
            (
                "ok",
                true,
                schema_bool_ir(
                    "twilio.schema.output.ok.title",
                    "twilio.schema.output.ok.description",
                ),
            ),
            (
                "receipt_id",
                false,
                schema_str(
                    "twilio.schema.output.receipt_id.title",
                    "twilio.schema.output.receipt_id.description",
                ),
            ),
        ],
        true,
    )
}

fn config_schema() -> SchemaIr {
    schema_obj(
        "twilio.schema.config.title",
        "twilio.schema.config.description",
        vec![
            (
                "enabled",
                true,
                schema_bool_ir(
                    "twilio.schema.config.enabled.title",
                    "twilio.schema.config.enabled.description",
                ),
            ),
            (
                "messaging_provider_id",
                true,
                schema_str(
                    "twilio.schema.config.messaging_provider_id.title",
                    "twilio.schema.config.messaging_provider_id.description",
                ),
            ),
            (
                "from",
                false,
                schema_str(
                    "twilio.schema.config.from.title",
                    "twilio.schema.config.from.description",
                ),
            ),
            (
                "persistence_key_prefix",
                false,
                schema_str(
                    "twilio.schema.config.persistence_key_prefix.title",
                    "twilio.schema.config.persistence_key_prefix.description",
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
