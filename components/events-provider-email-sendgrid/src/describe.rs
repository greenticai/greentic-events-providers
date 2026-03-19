//! Provider description and QA specs for SendGrid email event provider.

use provider_common::component_v0_6::{
    DescribePayload, QaSpec, SchemaIr, canonical_cbor_bytes, schema_hash,
};
use provider_common::helpers::{op, schema_bool_ir, schema_obj, schema_str};
use serde_json::{Value, json};

use crate::{PROVIDER_ID, WORLD_ID};

/// I18n keys for the SendGrid email provider.
pub(crate) const I18N_KEYS: &[&str] = &[
    "sendgrid.op.ingest_http.title",
    "sendgrid.op.ingest_http.description",
    "sendgrid.op.publish.title",
    "sendgrid.op.publish.description",
    "sendgrid.schema.input.title",
    "sendgrid.schema.input.description",
    "sendgrid.schema.input.event.title",
    "sendgrid.schema.input.event.description",
    "sendgrid.schema.output.title",
    "sendgrid.schema.output.description",
    "sendgrid.schema.output.ok.title",
    "sendgrid.schema.output.ok.description",
    "sendgrid.schema.output.receipt_id.title",
    "sendgrid.schema.output.receipt_id.description",
    "sendgrid.schema.config.title",
    "sendgrid.schema.config.description",
    "sendgrid.schema.config.enabled.title",
    "sendgrid.schema.config.enabled.description",
    "sendgrid.schema.config.messaging_provider_id.title",
    "sendgrid.schema.config.messaging_provider_id.description",
    "sendgrid.schema.config.from.title",
    "sendgrid.schema.config.from.description",
    "sendgrid.schema.config.persistence_key_prefix.title",
    "sendgrid.schema.config.persistence_key_prefix.description",
    "sendgrid.qa.default.title",
    "sendgrid.qa.setup.title",
    "sendgrid.qa.upgrade.title",
    "sendgrid.qa.remove.title",
    "sendgrid.qa.setup.enabled",
    "sendgrid.qa.setup.messaging_provider_id",
    "sendgrid.qa.setup.from",
    "sendgrid.qa.setup.persistence_key_prefix",
];

/// Setup questions for the SendGrid email provider.
pub(crate) const SETUP_QUESTIONS: &[provider_common::helpers::QaQuestionDef] = &[
    ("enabled", "sendgrid.qa.setup.enabled", true),
    (
        "messaging_provider_id",
        "sendgrid.qa.setup.messaging_provider_id",
        true,
    ),
    ("from", "sendgrid.qa.setup.from", false),
    (
        "persistence_key_prefix",
        "sendgrid.qa.setup.persistence_key_prefix",
        false,
    ),
];

/// Keys for default mode (minimal required config).
pub(crate) const DEFAULT_KEYS: &[&str] = &["messaging_provider_id"];

/// I18n pairs for English locale.
pub(crate) const I18N_PAIRS: &[(&str, &str)] = &[
    ("sendgrid.op.ingest_http.title", "Ingest HTTP"),
    (
        "sendgrid.op.ingest_http.description",
        "Process incoming SendGrid email event",
    ),
    ("sendgrid.op.publish.title", "Publish"),
    (
        "sendgrid.op.publish.description",
        "Publish SendGrid email event",
    ),
    ("sendgrid.schema.input.title", "SendGrid email input"),
    (
        "sendgrid.schema.input.description",
        "Input for SendGrid email operations",
    ),
    ("sendgrid.schema.input.event.title", "Event"),
    ("sendgrid.schema.input.event.description", "Event payload"),
    ("sendgrid.schema.output.title", "SendGrid email output"),
    (
        "sendgrid.schema.output.description",
        "Result of SendGrid email operation",
    ),
    ("sendgrid.schema.output.ok.title", "Success"),
    (
        "sendgrid.schema.output.ok.description",
        "Whether operation succeeded",
    ),
    ("sendgrid.schema.output.receipt_id.title", "Receipt ID"),
    (
        "sendgrid.schema.output.receipt_id.description",
        "Unique identifier for the processed event",
    ),
    ("sendgrid.schema.config.title", "SendGrid email config"),
    (
        "sendgrid.schema.config.description",
        "SendGrid email provider configuration",
    ),
    ("sendgrid.schema.config.enabled.title", "Enabled"),
    (
        "sendgrid.schema.config.enabled.description",
        "Enable this provider",
    ),
    (
        "sendgrid.schema.config.messaging_provider_id.title",
        "Messaging Provider ID",
    ),
    (
        "sendgrid.schema.config.messaging_provider_id.description",
        "ID of the messaging provider for email delivery",
    ),
    ("sendgrid.schema.config.from.title", "From Address"),
    (
        "sendgrid.schema.config.from.description",
        "Default sender email address",
    ),
    (
        "sendgrid.schema.config.persistence_key_prefix.title",
        "Persistence Key Prefix",
    ),
    (
        "sendgrid.schema.config.persistence_key_prefix.description",
        "Prefix for state store keys",
    ),
    ("sendgrid.qa.default.title", "Default"),
    ("sendgrid.qa.setup.title", "Setup"),
    ("sendgrid.qa.upgrade.title", "Upgrade"),
    ("sendgrid.qa.remove.title", "Remove"),
    ("sendgrid.qa.setup.enabled", "Enable provider"),
    (
        "sendgrid.qa.setup.messaging_provider_id",
        "Messaging Provider ID",
    ),
    ("sendgrid.qa.setup.from", "From Address"),
    (
        "sendgrid.qa.setup.persistence_key_prefix",
        "Persistence Key Prefix",
    ),
];

/// Build the describe payload for the SendGrid email provider.
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
                "sendgrid.op.ingest_http.title",
                "sendgrid.op.ingest_http.description",
            ),
            op(
                "publish",
                "sendgrid.op.publish.title",
                "sendgrid.op.publish.description",
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
    provider_common::helpers::qa_spec_for_mode(mode_str, "sendgrid", SETUP_QUESTIONS, DEFAULT_KEYS)
}

fn input_schema() -> SchemaIr {
    schema_obj(
        "sendgrid.schema.input.title",
        "sendgrid.schema.input.description",
        vec![(
            "event",
            true,
            schema_str(
                "sendgrid.schema.input.event.title",
                "sendgrid.schema.input.event.description",
            ),
        )],
        true,
    )
}

fn output_schema() -> SchemaIr {
    schema_obj(
        "sendgrid.schema.output.title",
        "sendgrid.schema.output.description",
        vec![
            (
                "ok",
                true,
                schema_bool_ir(
                    "sendgrid.schema.output.ok.title",
                    "sendgrid.schema.output.ok.description",
                ),
            ),
            (
                "receipt_id",
                false,
                schema_str(
                    "sendgrid.schema.output.receipt_id.title",
                    "sendgrid.schema.output.receipt_id.description",
                ),
            ),
        ],
        true,
    )
}

fn config_schema() -> SchemaIr {
    schema_obj(
        "sendgrid.schema.config.title",
        "sendgrid.schema.config.description",
        vec![
            (
                "enabled",
                true,
                schema_bool_ir(
                    "sendgrid.schema.config.enabled.title",
                    "sendgrid.schema.config.enabled.description",
                ),
            ),
            (
                "messaging_provider_id",
                true,
                schema_str(
                    "sendgrid.schema.config.messaging_provider_id.title",
                    "sendgrid.schema.config.messaging_provider_id.description",
                ),
            ),
            (
                "from",
                false,
                schema_str(
                    "sendgrid.schema.config.from.title",
                    "sendgrid.schema.config.from.description",
                ),
            ),
            (
                "persistence_key_prefix",
                false,
                schema_str(
                    "sendgrid.schema.config.persistence_key_prefix.title",
                    "sendgrid.schema.config.persistence_key_prefix.description",
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
