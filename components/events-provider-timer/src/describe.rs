//! Provider description and QA specs for timer event provider.

use provider_common::component_v0_6::{
    DescribePayload, QaSpec, SchemaIr, canonical_cbor_bytes, schema_hash,
};
use provider_common::helpers::{op, schema_bool_ir, schema_number_ir, schema_obj, schema_str};
use serde_json::{Value, json};

use crate::{PROVIDER_ID, WORLD_ID};

/// I18n keys for the timer provider.
pub(crate) const I18N_KEYS: &[&str] = &[
    "timer.op.timer_tick.title",
    "timer.op.timer_tick.description",
    "timer.op.publish.title",
    "timer.op.publish.description",
    "timer.schema.input.title",
    "timer.schema.input.description",
    "timer.schema.input.event.title",
    "timer.schema.input.event.description",
    "timer.schema.output.title",
    "timer.schema.output.description",
    "timer.schema.output.ok.title",
    "timer.schema.output.ok.description",
    "timer.schema.output.receipt_id.title",
    "timer.schema.output.receipt_id.description",
    "timer.schema.config.title",
    "timer.schema.config.description",
    "timer.schema.config.enabled.title",
    "timer.schema.config.enabled.description",
    "timer.schema.config.timezone.title",
    "timer.schema.config.timezone.description",
    "timer.schema.config.default_delay_seconds.title",
    "timer.schema.config.default_delay_seconds.description",
    "timer.schema.config.persistence_key_prefix.title",
    "timer.schema.config.persistence_key_prefix.description",
    "timer.qa.default.title",
    "timer.qa.setup.title",
    "timer.qa.upgrade.title",
    "timer.qa.remove.title",
    "timer.qa.setup.enabled",
    "timer.qa.setup.timezone",
    "timer.qa.setup.default_delay_seconds",
    "timer.qa.setup.persistence_key_prefix",
];

/// Setup questions for the timer provider.
pub(crate) const SETUP_QUESTIONS: &[provider_common::helpers::QaQuestionDef] = &[
    ("enabled", "timer.qa.setup.enabled", true),
    ("timezone", "timer.qa.setup.timezone", false),
    (
        "default_delay_seconds",
        "timer.qa.setup.default_delay_seconds",
        false,
    ),
    (
        "persistence_key_prefix",
        "timer.qa.setup.persistence_key_prefix",
        false,
    ),
];

/// Keys for default mode (minimal required config).
pub(crate) const DEFAULT_KEYS: &[&str] = &[];

/// I18n pairs for English locale.
pub(crate) const I18N_PAIRS: &[(&str, &str)] = &[
    ("timer.op.timer_tick.title", "Timer Tick"),
    (
        "timer.op.timer_tick.description",
        "Process timer tick event",
    ),
    ("timer.op.publish.title", "Publish"),
    ("timer.op.publish.description", "Publish timer event"),
    ("timer.schema.input.title", "Timer input"),
    (
        "timer.schema.input.description",
        "Input for timer operations",
    ),
    ("timer.schema.input.event.title", "Event"),
    ("timer.schema.input.event.description", "Event payload"),
    ("timer.schema.output.title", "Timer output"),
    (
        "timer.schema.output.description",
        "Result of timer operation",
    ),
    ("timer.schema.output.ok.title", "Success"),
    (
        "timer.schema.output.ok.description",
        "Whether operation succeeded",
    ),
    ("timer.schema.output.receipt_id.title", "Receipt ID"),
    (
        "timer.schema.output.receipt_id.description",
        "Unique identifier for the processed event",
    ),
    ("timer.schema.config.title", "Timer config"),
    (
        "timer.schema.config.description",
        "Timer provider configuration",
    ),
    ("timer.schema.config.enabled.title", "Enabled"),
    (
        "timer.schema.config.enabled.description",
        "Enable this provider",
    ),
    ("timer.schema.config.timezone.title", "Timezone"),
    (
        "timer.schema.config.timezone.description",
        "Timezone for timer events (e.g. UTC, America/New_York)",
    ),
    (
        "timer.schema.config.default_delay_seconds.title",
        "Default Delay (seconds)",
    ),
    (
        "timer.schema.config.default_delay_seconds.description",
        "Default delay in seconds before timer fires",
    ),
    (
        "timer.schema.config.persistence_key_prefix.title",
        "Persistence Key Prefix",
    ),
    (
        "timer.schema.config.persistence_key_prefix.description",
        "Prefix for state store keys",
    ),
    ("timer.qa.default.title", "Default"),
    ("timer.qa.setup.title", "Setup"),
    ("timer.qa.upgrade.title", "Upgrade"),
    ("timer.qa.remove.title", "Remove"),
    ("timer.qa.setup.enabled", "Enable provider"),
    ("timer.qa.setup.timezone", "Timezone"),
    (
        "timer.qa.setup.default_delay_seconds",
        "Default Delay (seconds)",
    ),
    (
        "timer.qa.setup.persistence_key_prefix",
        "Persistence Key Prefix",
    ),
];

/// Build the describe payload for the timer provider.
pub(crate) fn build_describe_payload() -> DescribePayload {
    let input_schema = input_schema();
    let output_schema = output_schema();
    let config_schema = config_schema();
    DescribePayload {
        provider: PROVIDER_ID.to_string(),
        world: WORLD_ID.to_string(),
        operations: vec![
            op(
                "timer_tick",
                "timer.op.timer_tick.title",
                "timer.op.timer_tick.description",
            ),
            op(
                "publish",
                "timer.op.publish.title",
                "timer.op.publish.description",
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
    let mut spec = provider_common::helpers::qa_spec_for_mode(
        mode_str,
        "timer",
        SETUP_QUESTIONS,
        DEFAULT_KEYS,
    );

    // Set default values for optional fields
    for q in &mut spec.questions {
        if q.id == "timezone" && q.default.is_none() {
            q.default = Some(serde_json::Value::String("UTC".to_string()));
        }
        if q.id == "default_delay_seconds" && q.default.is_none() {
            q.default = Some(serde_json::Value::Number(30.into()));
        }
        if q.id == "persistence_key_prefix" && q.default.is_none() {
            q.default = Some(serde_json::Value::String(
                "events/timer/scheduled".to_string(),
            ));
        }
    }
    spec
}

fn input_schema() -> SchemaIr {
    schema_obj(
        "timer.schema.input.title",
        "timer.schema.input.description",
        vec![(
            "event",
            true,
            schema_str(
                "timer.schema.input.event.title",
                "timer.schema.input.event.description",
            ),
        )],
        true,
    )
}

fn output_schema() -> SchemaIr {
    schema_obj(
        "timer.schema.output.title",
        "timer.schema.output.description",
        vec![
            (
                "ok",
                true,
                schema_bool_ir(
                    "timer.schema.output.ok.title",
                    "timer.schema.output.ok.description",
                ),
            ),
            (
                "receipt_id",
                false,
                schema_str(
                    "timer.schema.output.receipt_id.title",
                    "timer.schema.output.receipt_id.description",
                ),
            ),
        ],
        true,
    )
}

fn config_schema() -> SchemaIr {
    schema_obj(
        "timer.schema.config.title",
        "timer.schema.config.description",
        vec![
            (
                "enabled",
                true,
                schema_bool_ir(
                    "timer.schema.config.enabled.title",
                    "timer.schema.config.enabled.description",
                ),
            ),
            (
                "timezone",
                false,
                schema_str(
                    "timer.schema.config.timezone.title",
                    "timer.schema.config.timezone.description",
                ),
            ),
            (
                "default_delay_seconds",
                false,
                schema_number_ir(
                    "timer.schema.config.default_delay_seconds.title",
                    "timer.schema.config.default_delay_seconds.description",
                ),
            ),
            (
                "persistence_key_prefix",
                false,
                schema_str(
                    "timer.schema.config.persistence_key_prefix.title",
                    "timer.schema.config.persistence_key_prefix.description",
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
