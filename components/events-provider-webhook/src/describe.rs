//! Provider description and QA specs for webhook event provider.

use provider_common::component_v0_6::{
    DescribePayload, QaSpec, RedactionRule, SchemaIr, canonical_cbor_bytes, schema_hash,
};
use provider_common::helpers::{
    op, schema_bool_ir, schema_obj, schema_secret, schema_str, schema_str_fmt,
};
use serde_json::{Value, json};

use crate::{PROVIDER_ID, WORLD_ID};

/// I18n keys for the webhook provider.
pub(crate) const I18N_KEYS: &[&str] = &[
    "webhook.op.ingest_http.title",
    "webhook.op.ingest_http.description",
    "webhook.op.publish.title",
    "webhook.op.publish.description",
    "webhook.schema.input.title",
    "webhook.schema.input.description",
    "webhook.schema.input.event.title",
    "webhook.schema.input.event.description",
    "webhook.schema.output.title",
    "webhook.schema.output.description",
    "webhook.schema.output.ok.title",
    "webhook.schema.output.ok.description",
    "webhook.schema.output.receipt_id.title",
    "webhook.schema.output.receipt_id.description",
    "webhook.schema.config.title",
    "webhook.schema.config.description",
    "webhook.schema.config.enabled.title",
    "webhook.schema.config.enabled.description",
    "webhook.schema.config.target_url.title",
    "webhook.schema.config.target_url.description",
    "webhook.schema.config.method.title",
    "webhook.schema.config.method.description",
    "webhook.schema.config.auth_token.title",
    "webhook.schema.config.auth_token.description",
    "webhook.schema.config.timeout_ms.title",
    "webhook.schema.config.timeout_ms.description",
    "webhook.qa.default.title",
    "webhook.qa.setup.title",
    "webhook.qa.upgrade.title",
    "webhook.qa.remove.title",
    "webhook.qa.setup.enabled",
    "webhook.qa.setup.target_url",
    "webhook.qa.setup.method",
    "webhook.qa.setup.auth_token",
    "webhook.qa.setup.timeout_ms",
];

/// Setup questions for the webhook provider.
pub(crate) const SETUP_QUESTIONS: &[provider_common::helpers::QaQuestionDef] = &[
    ("enabled", "webhook.qa.setup.enabled", true),
    ("target_url", "webhook.qa.setup.target_url", true),
    ("method", "webhook.qa.setup.method", false),
    ("auth_token", "webhook.qa.setup.auth_token", false),
    ("timeout_ms", "webhook.qa.setup.timeout_ms", false),
];

/// Keys for default mode (minimal required config).
pub(crate) const DEFAULT_KEYS: &[&str] = &["target_url"];

/// I18n pairs for English locale.
pub(crate) const I18N_PAIRS: &[(&str, &str)] = &[
    ("webhook.op.ingest_http.title", "Ingest HTTP"),
    (
        "webhook.op.ingest_http.description",
        "Process incoming webhook event",
    ),
    ("webhook.op.publish.title", "Publish"),
    (
        "webhook.op.publish.description",
        "Publish event to webhook target",
    ),
    ("webhook.schema.input.title", "Webhook input"),
    (
        "webhook.schema.input.description",
        "Input for webhook operations",
    ),
    ("webhook.schema.input.event.title", "Event"),
    ("webhook.schema.input.event.description", "Event payload"),
    ("webhook.schema.output.title", "Webhook output"),
    (
        "webhook.schema.output.description",
        "Result of webhook operation",
    ),
    ("webhook.schema.output.ok.title", "Success"),
    (
        "webhook.schema.output.ok.description",
        "Whether operation succeeded",
    ),
    ("webhook.schema.output.receipt_id.title", "Receipt ID"),
    (
        "webhook.schema.output.receipt_id.description",
        "Unique identifier for the processed event",
    ),
    ("webhook.schema.config.title", "Webhook config"),
    (
        "webhook.schema.config.description",
        "Webhook provider configuration",
    ),
    ("webhook.schema.config.enabled.title", "Enabled"),
    (
        "webhook.schema.config.enabled.description",
        "Enable this provider",
    ),
    ("webhook.schema.config.target_url.title", "Target URL"),
    (
        "webhook.schema.config.target_url.description",
        "URL to send webhook events to",
    ),
    ("webhook.schema.config.method.title", "HTTP Method"),
    (
        "webhook.schema.config.method.description",
        "HTTP method (POST, PUT, etc.)",
    ),
    (
        "webhook.schema.config.auth_token.title",
        "Authorization Token",
    ),
    (
        "webhook.schema.config.auth_token.description",
        "Bearer token for webhook authentication",
    ),
    ("webhook.schema.config.timeout_ms.title", "Timeout (ms)"),
    (
        "webhook.schema.config.timeout_ms.description",
        "Request timeout in milliseconds",
    ),
    ("webhook.qa.default.title", "Default"),
    ("webhook.qa.setup.title", "Setup"),
    ("webhook.qa.upgrade.title", "Upgrade"),
    ("webhook.qa.remove.title", "Remove"),
    ("webhook.qa.setup.enabled", "Enable provider"),
    ("webhook.qa.setup.target_url", "Target URL"),
    ("webhook.qa.setup.method", "HTTP Method"),
    ("webhook.qa.setup.auth_token", "Authorization Token"),
    ("webhook.qa.setup.timeout_ms", "Timeout (ms)"),
];

/// Build the describe payload for the webhook provider.
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
                "webhook.op.ingest_http.title",
                "webhook.op.ingest_http.description",
            ),
            op(
                "publish",
                "webhook.op.publish.title",
                "webhook.op.publish.description",
            ),
        ],
        input_schema: input_schema.clone(),
        output_schema: output_schema.clone(),
        config_schema: config_schema.clone(),
        redactions: vec![RedactionRule {
            path: "$.auth_token".to_string(),
            strategy: "replace".to_string(),
        }],
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
    let mut spec = provider_common::helpers::qa_spec_for_mode(
        mode_str,
        "webhook",
        SETUP_QUESTIONS,
        DEFAULT_KEYS,
    );

    // Set default values for optional fields
    for q in &mut spec.questions {
        if q.id == "method" && q.default.is_none() {
            q.default = Some(serde_json::Value::String("POST".to_string()));
        }
        if q.id == "timeout_ms" && q.default.is_none() {
            q.default = Some(serde_json::Value::Number(5000.into()));
        }
    }
    spec
}

fn input_schema() -> SchemaIr {
    schema_obj(
        "webhook.schema.input.title",
        "webhook.schema.input.description",
        vec![(
            "event",
            true,
            schema_str(
                "webhook.schema.input.event.title",
                "webhook.schema.input.event.description",
            ),
        )],
        true,
    )
}

fn output_schema() -> SchemaIr {
    schema_obj(
        "webhook.schema.output.title",
        "webhook.schema.output.description",
        vec![
            (
                "ok",
                true,
                schema_bool_ir(
                    "webhook.schema.output.ok.title",
                    "webhook.schema.output.ok.description",
                ),
            ),
            (
                "receipt_id",
                false,
                schema_str(
                    "webhook.schema.output.receipt_id.title",
                    "webhook.schema.output.receipt_id.description",
                ),
            ),
        ],
        true,
    )
}

fn config_schema() -> SchemaIr {
    schema_obj(
        "webhook.schema.config.title",
        "webhook.schema.config.description",
        vec![
            (
                "enabled",
                true,
                schema_bool_ir(
                    "webhook.schema.config.enabled.title",
                    "webhook.schema.config.enabled.description",
                ),
            ),
            (
                "target_url",
                true,
                schema_str_fmt(
                    "webhook.schema.config.target_url.title",
                    "webhook.schema.config.target_url.description",
                    "uri",
                ),
            ),
            (
                "method",
                false,
                schema_str(
                    "webhook.schema.config.method.title",
                    "webhook.schema.config.method.description",
                ),
            ),
            (
                "auth_token",
                false,
                schema_secret(
                    "webhook.schema.config.auth_token.title",
                    "webhook.schema.config.auth_token.description",
                ),
            ),
            (
                "timeout_ms",
                false,
                schema_str(
                    "webhook.schema.config.timeout_ms.title",
                    "webhook.schema.config.timeout_ms.description",
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
