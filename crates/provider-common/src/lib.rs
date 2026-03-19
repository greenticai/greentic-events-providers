//! Common utilities for event provider components.
//!
//! Provides QA spec building, CBOR encoding, and helper functions
//! shared across all event providers.

pub mod component_v0_6;
pub mod helpers;
pub mod qa_invoke_bridge;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Common error type for event providers.
#[derive(Debug, Error, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProviderError {
    #[error("validation error: {0}")]
    Validation(String),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("missing secret: {name} (scope: {scope})")]
    MissingSecret {
        name: String,
        scope: String,
        remediation: String,
    },
    #[error("unknown provider error: {0}")]
    Other(String),
}

impl ProviderError {
    pub fn validation(msg: impl Into<String>) -> Self {
        ProviderError::Validation(msg.into())
    }

    pub fn transport(msg: impl Into<String>) -> Self {
        ProviderError::Transport(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        ProviderError::Other(msg.into())
    }

    pub fn missing_secret(name: impl Into<String>) -> Self {
        let name = name.into();
        ProviderError::MissingSecret {
            name: name.clone(),
            scope: "tenant".into(),
            remediation: format!(
                "Provide the `{name}` secret via greentic:secrets-store for this tenant."
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_validation_error() {
        let err = ProviderError::validation("missing token");
        assert_eq!(err, ProviderError::Validation("missing token".into()));
        assert_eq!(err.to_string(), "validation error: missing token");
    }

    #[test]
    fn builds_missing_secret_error() {
        let err = ProviderError::missing_secret("API_KEY");
        assert!(matches!(err, ProviderError::MissingSecret { .. }));
        assert_eq!(err.to_string(), "missing secret: API_KEY (scope: tenant)");
    }
}
