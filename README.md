# greentic-events-providers

Reusable Greentic event providers shipped as WASM components plus packs for `greentic-events` and `greentic-deployer`.

## What is here
- Provider families: webhook (HTTP in/out), email (MS Graph / Gmail), sms (Twilio), timer (cron/interval).
- WASM components implement `greentic:events@1.0.0` worlds via `greentic-interfaces-guest`.
- Packs under `packs/events` declare providers for discovery and deployment.
- Example flows live under `packs/events-*/flows` alongside each pack.

## Structure
- `crates/provider-core` – shared configs, helpers, errors.
- `crates/provider-webhook` – webhook source/sink mappings.
- `crates/provider-email` – email source/sink mappings for Graph/Gmail.
- `crates/provider-sms` – Twilio SMS source/sink.
- `crates/provider-timer` – cron/interval source.
- `docs/` – overview + per-provider notes.
- `packs/events/` – YAML packs consumed by greentic-events/deployer.
- `packs/events-*/flows/` – default/custom flow stubs referenced by packs.
- `scripts/build_packs.sh` – builds validated `*.gtpack` artifacts via `greentic-pack` (use `cargo binstall greentic-pack --locked`; optionally set `PACK_SERIES=0.4.` in CI to enforce a series).
- `.github/workflows/publish-packs.yaml` – builds `*.gtpack` with `greentic-pack` and publishes to GHCR on tags.
- `ci/local_check.sh` – run fmt + clippy + tests + pack build locally (mirrors CI).
- `.github/workflows/ci.yml` – CI for fmt/clippy/tests; live tests gated by vars; builds packs.
- `.github/workflows/publish-latest-packs.yaml` – publishes latest `*.gtpack` artifacts to GHCR on main.

## Versioning & constraints
- Rust edition 2024, MSRV 1.90.
- Depends on greentic crates at `0.4`.
- Components avoid hosting HTTP/timers; hosts feed requests/ticks into the WASM modules.

## Developing
- Build with `cargo build` (workspace).
- Run native tests with `cargo test`.
- Run wasm component lint/build through:
  - [ci/local_check.sh](/home/vgrishkyan/greentic/greentic-events-providers/ci/local_check.sh)
  - or explicit `cargo clippy --target wasm32-wasip2 ...`
- Packs are validated/built via `greentic-pack` (0.4.x) into `dist/packs/*.gtpack`; flows remain minimal placeholders but satisfy schema.

## Secrets workflow
- Packs declare `secret_requirements` inside component capabilities; run `scripts/build_packs.sh` then `greentic-secrets init --pack dist/packs/<pack>.gtpack` to provision required keys.
- Components resolve credentials through `greentic:secrets-store@1.0.0` only (no env/URI fallbacks in production paths) and return `secret_events` metadata alongside their main operations for hosts to forward.
- Secrets events use metadata-only payloads on standardized topics: `greentic.secrets.put`, `greentic.secrets.delete`, `greentic.secrets.rotate.requested`, `greentic.secrets.rotate.completed`, and `greentic.secrets.missing.detected`.
- Payloads include key/scope/tenant context and outcomes only—never secret bytes or base64 encodings.
- Host wiring examples live in `docs/hosts_secrets.md`.

## Integration testing against real services
The CI/pipeline can run live integration tests per provider when the relevant secrets are present. If a provider’s secrets are missing, tests should emit a warning and skip that provider.

Enable live tests by setting `RUN_LIVE_TESTS=true` and providing the env vars below.
Optionally set `RUN_LIVE_HTTP=true` to let the live tests make real HTTP calls (Graph token, Gmail token, Twilio API, webhook echo).

### Required secrets
- **Webhook**: none (local echo server used).
- **Email (Microsoft Graph)**:
  - `MSGRAPH_CLIENT_ID`
  - `MSGRAPH_CLIENT_SECRET`
  - `MSGRAPH_TENANT_ID`
  - `MSGRAPH_TEST_USER` (UPN/email to send/receive)
- **Email (Gmail/Workspace)**:
  - `GMAIL_CLIENT_ID`
  - `GMAIL_CLIENT_SECRET`
  - `GMAIL_REFRESH_TOKEN` (for test user)
  - `GMAIL_TEST_USER` (email to send/receive)
- **SMS (Twilio)**:
  - `TWILIO_ACCOUNT_SID`
  - `TWILIO_AUTH_TOKEN`
  - `TWILIO_FROM_NUMBER`
  - `TWILIO_TO_NUMBER` (destination for test send)
- **Timer**: none (pure logic).

### Behavior when secrets are missing
Integration test harnesses should:
- Detect missing env vars for each provider group.
- Print a clear warning indicating which provider tests were skipped and which secrets are required.
- Exit with success for skipped providers so CI does not fail when secrets are absent.

### CI
- `.github/workflows/ci.yml` runs unit tests on every push/PR.
- Live tests run when repository variable `RUN_LIVE_TESTS` is set to `true`; they rely on the secrets above and can make real HTTP calls if `RUN_LIVE_HTTP=true` is also set.
- The recommended first live-provider path is documented in:
  - [live_email_provider.md](/home/vgrishkyan/greentic/greentic-events-providers/docs/live_email_provider.md)
