# Security Fix Report

Date: 2026-03-31 (UTC)
Branch: `fix/oci-publish-path-events-subdir`

## Inputs Reviewed
- Security alerts JSON:
  - `dependabot`: `[]`
  - `code_scanning`: `[]`
- New PR Dependency Vulnerabilities: `[]`

## PR Dependency Change Review
Commands used:
- `cat pr-changed-files.txt`
- `git diff --name-only origin/main...HEAD -- Cargo.toml Cargo.lock '**/Cargo.toml' '**/package*.json' '**/requirements*.txt' '**/poetry.lock' '**/Pipfile*' '**/go.mod' '**/go.sum'`
- `git diff origin/main...HEAD -- Cargo.toml`
- `git diff origin/main...HEAD -- Cargo.lock`

Observed dependency file changes in PR scope:
- `Cargo.toml`: workspace package version changed from `0.4.13` to `0.4.14`.
- `Cargo.lock`: internal workspace crate versions changed from `0.4.13` to `0.4.14`.

Result:
- No third-party dependency additions or version upgrades were introduced.
- No new PR dependency vulnerabilities were identified.

## Remediation Actions
- No code or dependency remediation was required.
- No security fixes were applied because there are no active Dependabot alerts, no code scanning alerts, and no PR dependency vulnerabilities.

## Verification Notes
- Attempted runtime vulnerability scan with `cargo audit`, but it could not run in this CI sandbox due to Rust toolchain temp-file write restrictions under `/home/runner/.rustup`.
- Deterministic review was completed from provided alert artifacts and dependency diffs.

## Final Status
- `dependabot` alerts remediated: Not applicable (none present).
- `code_scanning` alerts remediated: Not applicable (none present).
- PR dependency vulnerabilities remediated: Not applicable (none present).
