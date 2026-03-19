//! Provider description and QA specs for dummy event provider.

use provider_common::component_v0_6::{
    DescribePayload, QaSpec, SchemaIr, canonical_cbor_bytes, schema_hash,
};
use provider_common::helpers::{op, schema_bool_ir, schema_obj, schema_str};
use serde_json::{Value, json};

use crate::{PROVIDER_ID, WORLD_ID};

/// I18n keys for the dummy provider.
pub(crate) const I18N_KEYS: &[&str] = &[
    "dummy.op.publish.title",
    "dummy.op.publish.description",
    "dummy.op.echo.title",
    "dummy.op.echo.description",
    "dummy.schema.input.title",
    "dummy.schema.input.description",
    "dummy.schema.input.payload.title",
    "dummy.schema.input.payload.description",
    "dummy.schema.output.title",
    "dummy.schema.output.description",
    "dummy.schema.output.ok.title",
    "dummy.schema.output.ok.description",
    "dummy.schema.output.receipt_id.title",
    "dummy.schema.output.receipt_id.description",
    "dummy.schema.config.title",
    "dummy.schema.config.description",
    "dummy.schema.config.enabled.title",
    "dummy.schema.config.enabled.description",
    "dummy.qa.default.title",
    "dummy.qa.setup.title",
    "dummy.qa.upgrade.title",
    "dummy.qa.remove.title",
    "dummy.qa.setup.enabled",
];

/// Setup questions for the dummy provider.
pub(crate) const SETUP_QUESTIONS: &[provider_common::helpers::QaQuestionDef] =
    &[("enabled", "dummy.qa.setup.enabled", true)];

/// Keys for default mode (minimal required config).
pub(crate) const DEFAULT_KEYS: &[&str] = &[];

/// I18n pairs for English locale.
pub(crate) const I18N_PAIRS: &[(&str, &str)] = &[
    ("dummy.op.publish.title", "Publish"),
    (
        "dummy.op.publish.description",
        "Publish event to dummy provider",
    ),
    ("dummy.op.echo.title", "Echo"),
    ("dummy.op.echo.description", "Echo back the input payload"),
    ("dummy.schema.input.title", "Dummy input"),
    (
        "dummy.schema.input.description",
        "Input for dummy operations",
    ),
    ("dummy.schema.input.payload.title", "Payload"),
    (
        "dummy.schema.input.payload.description",
        "Event payload (any JSON)",
    ),
    ("dummy.schema.output.title", "Dummy output"),
    (
        "dummy.schema.output.description",
        "Result of dummy operation",
    ),
    ("dummy.schema.output.ok.title", "Success"),
    (
        "dummy.schema.output.ok.description",
        "Whether operation succeeded",
    ),
    ("dummy.schema.output.receipt_id.title", "Receipt ID"),
    (
        "dummy.schema.output.receipt_id.description",
        "Unique identifier for the processed event",
    ),
    ("dummy.schema.config.title", "Dummy config"),
    (
        "dummy.schema.config.description",
        "Dummy provider configuration",
    ),
    ("dummy.schema.config.enabled.title", "Enabled"),
    (
        "dummy.schema.config.enabled.description",
        "Enable this provider",
    ),
    ("dummy.qa.default.title", "Default"),
    ("dummy.qa.setup.title", "Setup"),
    ("dummy.qa.upgrade.title", "Upgrade"),
    ("dummy.qa.remove.title", "Remove"),
    ("dummy.qa.setup.enabled", "Enable provider"),
];

/// Build the describe payload for the dummy provider.
pub(crate) fn build_describe_payload() -> DescribePayload {
    let input_schema = input_schema();
    let output_schema = output_schema();
    let config_schema = config_schema();
    DescribePayload {
        provider: PROVIDER_ID.to_string(),
        world: WORLD_ID.to_string(),
        operations: vec![
            op(
                "publish",
                "dummy.op.publish.title",
                "dummy.op.publish.description",
            ),
            op("echo", "dummy.op.echo.title", "dummy.op.echo.description"),
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
    provider_common::helpers::qa_spec_for_mode(mode_str, "dummy", SETUP_QUESTIONS, DEFAULT_KEYS)
}

fn input_schema() -> SchemaIr {
    schema_obj(
        "dummy.schema.input.title",
        "dummy.schema.input.description",
        vec![(
            "payload",
            false,
            schema_str(
                "dummy.schema.input.payload.title",
                "dummy.schema.input.payload.description",
            ),
        )],
        true,
    )
}

fn output_schema() -> SchemaIr {
    schema_obj(
        "dummy.schema.output.title",
        "dummy.schema.output.description",
        vec![
            (
                "ok",
                true,
                schema_bool_ir(
                    "dummy.schema.output.ok.title",
                    "dummy.schema.output.ok.description",
                ),
            ),
            (
                "receipt_id",
                false,
                schema_str(
                    "dummy.schema.output.receipt_id.title",
                    "dummy.schema.output.receipt_id.description",
                ),
            ),
        ],
        true,
    )
}

fn config_schema() -> SchemaIr {
    schema_obj(
        "dummy.schema.config.title",
        "dummy.schema.config.description",
        vec![(
            "enabled",
            true,
            schema_bool_ir(
                "dummy.schema.config.enabled.title",
                "dummy.schema.config.enabled.description",
            ),
        )],
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
