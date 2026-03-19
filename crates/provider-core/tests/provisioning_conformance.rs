use serde_json::Value;
use std::fs;
use std::path::Path;

fn read_pack_json(pack: &str) -> Value {
    let path = Path::new("../../packs").join(pack).join("pack.json");
    let raw = fs::read_to_string(&path).expect("pack.json exists");
    serde_json::from_str(&raw).expect("pack.json is valid json")
}

#[test]
fn pack_metadata_present() {
    let packs = [
        "events-email",
        "events-sms",
        "events-webhook",
        "events-timer",
        "events-email-sendgrid",
        "events-sms-twilio",
    ];

    for pack in packs {
        let value = read_pack_json(pack);
        let meta = value
            .get("meta")
            .unwrap_or_else(|| panic!("{pack} should have meta"));

        // Verify pack has required metadata fields
        assert!(
            meta.get("requires_public_base_url").is_some(),
            "{pack} should declare requires_public_base_url"
        );
        assert!(
            meta.get("capabilities").is_some(),
            "{pack} should declare capabilities"
        );

        // Verify no legacy entry_flows field
        assert!(
            meta.get("entry_flows").is_none(),
            "{pack} should not have legacy entry_flows"
        );
    }
}

#[test]
fn dummy_pack_has_no_setup() {
    let value = read_pack_json("events-dummy");
    let meta = value.get("meta").expect("meta present");

    // Verify no legacy entry_flows
    assert!(
        meta.get("entry_flows").is_none(),
        "events-dummy should not have legacy entry_flows"
    );

    let caps = meta
        .get("capabilities")
        .and_then(|v| v.as_array())
        .expect("capabilities present");
    assert!(
        caps.iter()
            .any(|cap| cap.as_str() == Some("provisioning:none")),
        "events-dummy should declare provisioning:none"
    );
}
