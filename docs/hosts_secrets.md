# Hosting components with secrets workflow

Hosts must wire provider components to the Greentic secrets store and forward metadata-only secret events alongside the main event flow.

## Secrets-store access
- Always resolve credentials via `greentic:secrets-store@1.0.0`.
- Do not inject env vars or URI-style secret paths into components.
- Use runtime `TenantCtx` to scope lookups; never bake tenant ids into keys.

## Secret events fanout
- Components now surface metadata-only `secret_events` on ingress/egress helpers.
- Hosts should publish these events on the event bus before/with the primary event and never include secret bytes.

## Integration tips

### Webhook source
```rust
let result = webhook_source.handle_request(tenant_ctx, inbound_request, &secrets_store)?;
for evt in result.secret_events {
    bus.publish(evt)?;
}
bus.publish(result.event)?;
```

### Email send
```rust
let req = provider_email::build_send_request(&envelope, &secrets_store)?;
for evt in req.secret_events.iter() {
    bus.publish(evt.clone())?;
}
let oauth = req.oauth.as_ref().expect("email providers should declare oauth hints");
let token = oauth_service.get_access_token(tenant_ctx.clone(), &oauth.provider_id, oauth.scopes.clone())?;
host_execute_email(req.payload)?;
```

### SMS send
```rust
let req = provider_sms::build_send_request(&cfg, &envelope, &secrets_store)?;
for evt in req.secret_events.iter() {
    bus.publish(evt.clone())?;
}
host_execute_twilio(req)?;
```

### Secrets store provider
- For wasm components, use `provider_core::secrets::SecretsStoreProvider` as the `SecretProvider` to resolve via the Greentic host surface.
- For host-side tests, `StaticSecretProvider` can stub required keys.

## Topics and payloads
- Standard topics: `greentic.secrets.put|delete|rotate.requested|rotate.completed|missing.detected`.
- Payloads carry schema_version/key/scope/tenant_ctx/result/timestamp (and rotation_id/error when applicable). Never include secret values or encodings.
