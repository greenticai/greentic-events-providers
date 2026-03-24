# Email provider

Purpose: inbound/outbound email events through Microsoft Graph or Gmail/Google Workspace.

- Component IDs: `events-email-source@1.0.0`, `events-email-sink@1.0.0`.
- Topics: inbound `email.in.msgraph.<folder>` / `email.in.gmail.<label>`; outbound `email.out.msgraph` / `email.out.gmail`.
- Inbound: host polls/receives mail and passes normalized email JSON; component emits `EventEnvelope`.
- Outbound: component builds provider-specific send payloads; host executes HTTP/SMTP. `EmailSendRequest` includes `secret_events` for hosts to forward before/alongside the send.
- Outbound: component builds provider-specific send payloads; host executes HTTP/SMTP. `EmailSendRequest` includes `secret_events` plus explicit OAuth token hints for host-side execution.
- Secrets/tokens: secrets provisioned via `greentic-secrets` using requirements in the pack (`MSGRAPH_CLIENT_SECRET`, `GMAIL_CLIENT_SECRET`, `GMAIL_REFRESH_TOKEN`); components read via `greentic:secrets-store@1.0.0` (no env fallbacks).
- Secrets events: emit metadata-only payloads on `greentic.secrets.put|delete|rotate.*|missing.detected` when secrets are resolved or missing (no values).
- Packs: `packs/events-email/pack.yaml`.
- Flows: `packs/events-email/flows/in_default.ygtc`, `packs/events-email/flows/out_default.ygtc`.

## Setup

- Entry flow: `setup` (collect → validate → apply → summary).
- Required inputs: `msgraph.client_id`, `gmail.client_id`, `sender.default_from`.
- Required secrets: `MSGRAPH_CLIENT_SECRET`, `GMAIL_CLIENT_SECRET`, `GMAIL_REFRESH_TOKEN`.
- Dry-run plan: emits subscription ops for MS Graph + Gmail; no webhook ops.

## Live-provider proof

The recommended first real provider proof is:

- outbound `email.out.msgraph` first
- outbound `email.out.gmail` second

See:

- [live_email_provider.md](/home/vgrishkyan/greentic/greentic-events-providers/docs/live_email_provider.md)
