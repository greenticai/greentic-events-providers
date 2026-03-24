# Live Email Provider Proof

This is the recommended first live-provider proof for
`greentic-events-providers`.

Use the email provider first:

- `events-email`
- outbound `MS Graph` first
- outbound `Gmail` second

## Why this is the right first live path

- the repo already ships `events-email` source/sink components
- the pack already exposes:
  - `email-out.msgraph`
  - `email-out.gmail`
- the live test harness already contains smoke tests for:
  - Microsoft Graph
  - Gmail
- OAuth-backed provider access is already the documented design boundary for
  this repo

References:

- [email.md](/home/vgrishkyan/greentic/greentic-events-providers/docs/email.md)
- [overview.md](/home/vgrishkyan/greentic/greentic-events-providers/docs/overview.md)
- [provider_tokens.md](/home/vgrishkyan/greentic/greentic-oauth/docs/provider_tokens.md)

## Current proof assets already in the repo

Implementation:

- [crates/provider-email/src/lib.rs](/home/vgrishkyan/greentic/greentic-events-providers/crates/provider-email/src/lib.rs)

Live smoke tests:

- [integration_live.rs](/home/vgrishkyan/greentic/greentic-events-providers/crates/provider-email/tests/integration_live.rs)

Pack:

- [pack.yaml](/home/vgrishkyan/greentic/greentic-events-providers/packs/events-email/pack.yaml)

CI:

- [ci.yml](/home/vgrishkyan/greentic/greentic-events-providers/.github/workflows/ci.yml)

## Recommended first proof

Run the first real live proof as:

1. AWS combined baseline already up
2. secrets-backed OAuth client config available
3. `events-email` outbound MS Graph path enabled
4. live HTTP token acquisition and outbound send exercised
5. runtime/admin checks still pass after the flow

This is the smallest provider path that proves:

- secrets
- oauth
- events
- telemetry
- cloud runtime

in one connected flow.

## Required live env for repo-local smoke

For Microsoft Graph:

- `RUN_LIVE_TESTS=true`
- `MSGRAPH_CLIENT_ID`
- `MSGRAPH_CLIENT_SECRET`
- `MSGRAPH_TENANT_ID`
- `MSGRAPH_TEST_USER`

Optional:

- `RUN_LIVE_HTTP=true`

For Gmail:

- `GMAIL_CLIENT_ID`
- `GMAIL_CLIENT_SECRET`
- `GMAIL_REFRESH_TOKEN`
- `GMAIL_TEST_USER`

## Expected behavior

- if `RUN_LIVE_TESTS` is not enabled, live tests skip cleanly
- if required provider secrets are missing, the provider-specific live test
  skips cleanly
- if `RUN_LIVE_HTTP=true`, the harness performs the real token + outbound HTTP
  call
- otherwise the harness still proves the routing/build path without network

## Recommended rollout order

1. outbound MS Graph
2. outbound Gmail
3. inbound Graph/Gmail subscription or polling flows
4. mirror the same proof shape onto Azure and GCP combined scenarios

This order gives the fastest honest proof without pretending that inbound,
subscriptions, and webhook paths are already the first thing to stabilize.
