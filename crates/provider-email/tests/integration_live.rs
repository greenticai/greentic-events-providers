use base64::Engine as _;
use provider_email::{EmailProvider, InboundEmail, build_send_request, map_inbound_email};
use std::collections::BTreeMap;
use std::env;
use std::error::Error;

fn should_run() -> bool {
    matches!(env::var("RUN_LIVE_TESTS"), Ok(val) if val == "true")
}

fn should_call_network() -> bool {
    matches!(env::var("RUN_LIVE_HTTP"), Ok(val) if val == "true")
}

fn collect_env(required: &[&str]) -> Option<BTreeMap<String, String>> {
    let mut missing = Vec::new();
    let mut vars = BTreeMap::new();
    for key in required {
        match env::var(key) {
            Ok(val) if !val.is_empty() => {
                vars.insert(key.to_string(), val);
            }
            _ => missing.push(*key),
        }
    }
    if missing.is_empty() {
        Some(vars)
    } else {
        eprintln!(
            "Skipping live test; missing env vars: {}",
            missing.join(", ")
        );
        None
    }
}

fn sample_tenant() -> greentic_types::TenantCtx {
    use greentic_types::{EnvId, TenantCtx, TenantId};
    let env = EnvId::try_from("dev").unwrap();
    let tenant = TenantId::try_from("live-tests").unwrap();
    TenantCtx::new(env, tenant)
}

#[test]
fn live_msgraph_integration_smoke() -> Result<(), Box<dyn Error>> {
    if !should_run() {
        eprintln!("Skipping live Graph test; set RUN_LIVE_TESTS=true to enable.");
        return Ok(());
    }
    let vars = match collect_env(&[
        "MSGRAPH_CLIENT_ID",
        "MSGRAPH_CLIENT_SECRET",
        "MSGRAPH_TENANT_ID",
        "MSGRAPH_TEST_USER",
    ]) {
        Some(v) => v,
        None => return Ok(()),
    };

    // Optional: hit Graph token endpoint to prove credentials work.
    if should_call_network() {
        let client = reqwest::blocking::Client::new();
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            vars["MSGRAPH_TENANT_ID"]
        );
        let token_res: serde_json::Value = client
            .post(token_url)
            .form(&[
                ("client_id", vars["MSGRAPH_CLIENT_ID"].as_str()),
                ("client_secret", vars["MSGRAPH_CLIENT_SECRET"].as_str()),
                ("grant_type", "client_credentials"),
                ("scope", "https://graph.microsoft.com/.default"),
            ])
            .send()?
            .json()?;
        if let Some(access_token) = token_res.get("access_token").and_then(|v| v.as_str()) {
            let send_url = format!(
                "https://graph.microsoft.com/v1.0/users/{}/sendMail",
                vars["MSGRAPH_TEST_USER"]
            );
            let send_body = serde_json::json!({
                "message": {
                    "subject": "Greentic live Graph test",
                    "body": { "contentType": "Text", "content": "This is a live Graph test message." },
                    "toRecipients": [{
                        "emailAddress": { "address": vars["MSGRAPH_TEST_USER"] }
                    }]
                },
                "saveToSentItems": false
            });
            let res = client
                .post(send_url)
                .bearer_auth(access_token)
                .json(&send_body)
                .send()?;
            if !res.status().is_success() {
                return Err(format!("Graph sendMail failed: {}", res.status()).into());
            }
        } else {
            return Err("Graph token response missing access_token".into());
        }
    } else {
        eprintln!("RUN_LIVE_HTTP not set; skipping outbound Graph HTTP call.");
    }

    // Smoke: map a synthetic inbound message; in real runs, this would pull via Graph.
    let inbound = InboundEmail {
        provider: EmailProvider::MsGraph,
        folder_or_label: "inbox".into(),
        message_id: "live-msg-graph".into(),
        subject: "Live Graph smoke".into(),
        from: vars["MSGRAPH_TEST_USER"].clone(),
        to: vec![vars["MSGRAPH_TEST_USER"].clone()],
        cc: vec![],
        bcc: vec![],
        received_at: chrono::Utc::now(),
        body: "This is a live smoke test".into(),
        headers: BTreeMap::new(),
    };
    let event = map_inbound_email(sample_tenant(), &inbound);
    assert!(event.topic.starts_with("email.in.msgraph"));
    Ok(())
}

#[test]
fn live_gmail_integration_smoke() -> Result<(), Box<dyn Error>> {
    if !should_run() {
        eprintln!("Skipping live Gmail test; set RUN_LIVE_TESTS=true to enable.");
        return Ok(());
    }
    let vars = match collect_env(&[
        "GMAIL_CLIENT_ID",
        "GMAIL_CLIENT_SECRET",
        "GMAIL_REFRESH_TOKEN",
        "GMAIL_TEST_USER",
    ]) {
        Some(v) => v,
        None => return Ok(()),
    };

    if should_call_network() {
        let client = reqwest::blocking::Client::new();
        let token_res: serde_json::Value = client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", vars["GMAIL_CLIENT_ID"].as_str()),
                ("client_secret", vars["GMAIL_CLIENT_SECRET"].as_str()),
                ("refresh_token", vars["GMAIL_REFRESH_TOKEN"].as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()?
            .json()?;

        if let Some(access_token) = token_res.get("access_token").and_then(|v| v.as_str()) {
            let raw_message = format!(
                "From: {}\r\nTo: {}\r\nSubject: Greentic live Gmail test\r\n\r\nThis is a live Gmail test message.",
                vars["GMAIL_TEST_USER"], vars["GMAIL_TEST_USER"]
            );
            let encoded =
                base64::engine::general_purpose::STANDARD_NO_PAD.encode(raw_message.as_bytes());
            let res = client
                .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
                .bearer_auth(access_token)
                .json(&serde_json::json!({ "raw": encoded }))
                .send()?;
            if !res.status().is_success() {
                return Err(format!("Gmail send failed: {}", res.status()).into());
            }
        } else {
            return Err("Gmail token response missing access_token".into());
        }
    } else {
        eprintln!("RUN_LIVE_HTTP not set; skipping outbound Gmail HTTP call.");
    }

    // Smoke: build an outbound send request envelope and ensure routing works.
    let payload = serde_json::json!({
        "to": [vars["GMAIL_TEST_USER"].clone()],
        "subject": "Live Gmail smoke",
        "body": "This is a live Gmail smoke test",
    });
    let envelope = greentic_types::EventEnvelope {
        id: greentic_types::EventId::new("live-gmail-1")?,
        topic: "email.out.gmail".into(),
        r#type: "com.greentic.email.generic.v1".into(),
        source: "integration-test".into(),
        tenant: sample_tenant(),
        subject: Some("Live Gmail smoke".into()),
        time: chrono::Utc::now(),
        correlation_id: None,
        payload: payload.clone(),
        metadata: BTreeMap::new(),
    };

    let secrets = provider_core::secrets::StaticSecretProvider::new(BTreeMap::from([
        (
            "GMAIL_CLIENT_SECRET".into(),
            vars["GMAIL_CLIENT_SECRET"].as_bytes().to_vec(),
        ),
        (
            "GMAIL_REFRESH_TOKEN".into(),
            vars["GMAIL_REFRESH_TOKEN"].as_bytes().to_vec(),
        ),
    ]));

    let request = build_send_request(&envelope, &secrets)?;
    assert_eq!(request.provider, EmailProvider::Gmail);
    assert_eq!(
        request.oauth,
        Some(provider_email::EmailOauthHint {
            provider_id: "gmail-email".into(),
            flow: "refresh_token".into(),
            scopes: vec!["https://www.googleapis.com/auth/gmail.send".into()],
        })
    );
    Ok(())
}
