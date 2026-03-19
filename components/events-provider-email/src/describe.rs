//! Provider description and QA specs for email event provider.

use provider_common::component_v0_6::{
    DescribePayload, QaSpec, SchemaIr, canonical_cbor_bytes, schema_hash,
};
use provider_common::helpers::{op, schema_bool_ir, schema_obj, schema_str};
use serde_json::{Value, json};

use crate::{PROVIDER_ID, WORLD_ID};

/// I18n keys for the email provider.
pub(crate) const I18N_KEYS: &[&str] = &[
    "email.op.ingest_http.title",
    "email.op.ingest_http.description",
    "email.op.publish.title",
    "email.op.publish.description",
    "email.schema.input.title",
    "email.schema.input.description",
    "email.schema.input.event.title",
    "email.schema.input.event.description",
    "email.schema.output.title",
    "email.schema.output.description",
    "email.schema.output.ok.title",
    "email.schema.output.ok.description",
    "email.schema.output.receipt_id.title",
    "email.schema.output.receipt_id.description",
    "email.schema.config.title",
    "email.schema.config.description",
    "email.schema.config.enabled.title",
    "email.schema.config.enabled.description",
    "email.schema.config.messaging_provider_id.title",
    "email.schema.config.messaging_provider_id.description",
    "email.schema.config.from.title",
    "email.schema.config.from.description",
    "email.schema.config.persistence_key_prefix.title",
    "email.schema.config.persistence_key_prefix.description",
    "email.qa.default.title",
    "email.qa.setup.title",
    "email.qa.upgrade.title",
    "email.qa.remove.title",
    "email.qa.setup.enabled",
    "email.qa.setup.messaging_provider_id",
    "email.qa.setup.from",
    "email.qa.setup.persistence_key_prefix",
];

/// Setup questions for the email provider.
pub(crate) const SETUP_QUESTIONS: &[provider_common::helpers::QaQuestionDef] = &[
    ("enabled", "email.qa.setup.enabled", true),
    (
        "messaging_provider_id",
        "email.qa.setup.messaging_provider_id",
        true,
    ),
    ("from", "email.qa.setup.from", false),
    (
        "persistence_key_prefix",
        "email.qa.setup.persistence_key_prefix",
        false,
    ),
];

/// Keys for default mode (minimal required config).
pub(crate) const DEFAULT_KEYS: &[&str] = &["messaging_provider_id"];

/// I18n pairs for English locale.
pub(crate) const I18N_PAIRS: &[(&str, &str)] = &[
    ("email.op.ingest_http.title", "Ingest HTTP"),
    (
        "email.op.ingest_http.description",
        "Process incoming email event",
    ),
    ("email.op.publish.title", "Publish"),
    ("email.op.publish.description", "Publish email event"),
    ("email.schema.input.title", "Email input"),
    (
        "email.schema.input.description",
        "Input for email operations",
    ),
    ("email.schema.input.event.title", "Event"),
    ("email.schema.input.event.description", "Event payload"),
    ("email.schema.output.title", "Email output"),
    (
        "email.schema.output.description",
        "Result of email operation",
    ),
    ("email.schema.output.ok.title", "Success"),
    (
        "email.schema.output.ok.description",
        "Whether operation succeeded",
    ),
    ("email.schema.output.receipt_id.title", "Receipt ID"),
    (
        "email.schema.output.receipt_id.description",
        "Unique identifier for the processed event",
    ),
    ("email.schema.config.title", "Email config"),
    (
        "email.schema.config.description",
        "Email provider configuration",
    ),
    ("email.schema.config.enabled.title", "Enabled"),
    (
        "email.schema.config.enabled.description",
        "Enable this provider",
    ),
    (
        "email.schema.config.messaging_provider_id.title",
        "Messaging Provider ID",
    ),
    (
        "email.schema.config.messaging_provider_id.description",
        "ID of the messaging provider for email delivery",
    ),
    ("email.schema.config.from.title", "From Address"),
    (
        "email.schema.config.from.description",
        "Default sender email address",
    ),
    (
        "email.schema.config.persistence_key_prefix.title",
        "Persistence Key Prefix",
    ),
    (
        "email.schema.config.persistence_key_prefix.description",
        "Prefix for state store keys",
    ),
    ("email.qa.default.title", "Default"),
    ("email.qa.setup.title", "Setup"),
    ("email.qa.upgrade.title", "Upgrade"),
    ("email.qa.remove.title", "Remove"),
    ("email.qa.setup.enabled", "Enable provider"),
    (
        "email.qa.setup.messaging_provider_id",
        "Messaging Provider ID",
    ),
    ("email.qa.setup.from", "From Address"),
    (
        "email.qa.setup.persistence_key_prefix",
        "Persistence Key Prefix",
    ),
];

/// Build the describe payload for the email provider.
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
                "email.op.ingest_http.title",
                "email.op.ingest_http.description",
            ),
            op(
                "publish",
                "email.op.publish.title",
                "email.op.publish.description",
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
    provider_common::helpers::qa_spec_for_mode(mode_str, "email", SETUP_QUESTIONS, DEFAULT_KEYS)
}

fn input_schema() -> SchemaIr {
    schema_obj(
        "email.schema.input.title",
        "email.schema.input.description",
        vec![(
            "event",
            true,
            schema_str(
                "email.schema.input.event.title",
                "email.schema.input.event.description",
            ),
        )],
        true,
    )
}

fn output_schema() -> SchemaIr {
    schema_obj(
        "email.schema.output.title",
        "email.schema.output.description",
        vec![
            (
                "ok",
                true,
                schema_bool_ir(
                    "email.schema.output.ok.title",
                    "email.schema.output.ok.description",
                ),
            ),
            (
                "receipt_id",
                false,
                schema_str(
                    "email.schema.output.receipt_id.title",
                    "email.schema.output.receipt_id.description",
                ),
            ),
        ],
        true,
    )
}

fn config_schema() -> SchemaIr {
    schema_obj(
        "email.schema.config.title",
        "email.schema.config.description",
        vec![
            (
                "enabled",
                true,
                schema_bool_ir(
                    "email.schema.config.enabled.title",
                    "email.schema.config.enabled.description",
                ),
            ),
            (
                "messaging_provider_id",
                true,
                schema_str(
                    "email.schema.config.messaging_provider_id.title",
                    "email.schema.config.messaging_provider_id.description",
                ),
            ),
            (
                "from",
                false,
                schema_str(
                    "email.schema.config.from.title",
                    "email.schema.config.from.description",
                ),
            ),
            (
                "persistence_key_prefix",
                false,
                schema_str(
                    "email.schema.config.persistence_key_prefix.title",
                    "email.schema.config.persistence_key_prefix.description",
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
