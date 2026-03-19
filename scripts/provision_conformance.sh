#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v greentic-provision >/dev/null 2>&1; then
  echo "greentic-provision not found. Install with: cargo binstall greentic-provision --locked" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for provisioning conformance checks." >&2
  exit 1
fi

PACKS=(
  "events-email"
  "events-sms"
  "events-webhook"
  "events-timer"
  "events-dummy"
)

pack_id_for() {
  case "$1" in
    events-email) echo "greentic.events.email" ;;
    events-sms) echo "greentic.events.sms" ;;
    events-webhook) echo "greentic.events.webhook" ;;
    events-timer) echo "greentic.events.timer" ;;
    events-dummy) echo "greentic.events.provider.dummy" ;;
    *)
      echo "Unknown pack id mapping for $1" >&2
      exit 1
      ;;
  esac
}

requires_base_url_for() {
  case "$1" in
    events-sms|events-webhook) echo "true" ;;
    *) echo "false" ;;
  esac
}

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

for pack in "${PACKS[@]}"; do
  pack_dir="${ROOT_DIR}/packs/${pack}"
  fixtures_dir="${pack_dir}/fixtures"
  answers="${fixtures_dir}/setup.input.json"
  expected="${fixtures_dir}/setup.expected.plan.json"
  requirements_expected="${fixtures_dir}/requirements.expected.json"

  if [ ! -f "${pack_dir}/pack.json" ]; then
    echo "Missing pack.json for ${pack_dir}" >&2
    exit 1
  fi

  # Check for provisioning:none capability first (skip setup validation)
  has_no_setup="$(PACK_DIR="${pack_dir}" python3 - <<'PY'
import json
import os
from pathlib import Path

pack_json = Path(os.environ["PACK_DIR"]) / "pack.json"
data = json.loads(pack_json.read_text())
caps = data.get("meta", {}).get("capabilities", [])
print("yes" if "provisioning:none" in caps else "no")
PY
)"
  if [ "${has_no_setup}" = "yes" ]; then
    echo "Pack ${pack} has provisioning:none capability, skipping setup validation."
    continue
  fi

  # For packs with setup flows, check required fixtures
  if [ ! -f "${requirements_expected}" ]; then
    echo "Missing requirements.expected.json for ${pack}" >&2
    exit 1
  fi
  has_setup="$(PACK_DIR="${pack_dir}" python3 - <<'PY'
import json
import os
from pathlib import Path

pack_json = Path(os.environ["PACK_DIR"]) / "pack.json"
data = json.loads(pack_json.read_text())
entry_flows = data.get("meta", {}).get("entry_flows", {})
setup = None
if isinstance(entry_flows, dict):
    setup = entry_flows.get("setup")
elif isinstance(entry_flows, list):
    for flow in entry_flows:
        entry = flow.get("entry") or flow.get("name")
        if entry == "setup":
            setup = flow.get("id") or flow.get("flow_id") or flow.get("name")
            break
print("yes" if setup else "no")
PY
)"
  if [ "${has_setup}" != "yes" ]; then
    echo "Pack ${pack} has no setup entry and no provisioning:none capability." >&2
    exit 1
  fi

  requirements_actual="${TMP_DIR}/${pack}.requirements.json"
  PACK_DIR="${pack_dir}" python3 - <<'PY' > "${requirements_actual}"
import codecs
import json
import os
import re
from pathlib import Path

wat_path = Path(os.environ["PACK_DIR"]) / "setup_default__requirements.wat"
text = wat_path.read_text()
match = re.search(r'\(data \(i32\.const 0\) "(.*?)"\)', text, re.DOTALL)
if not match:
    raise SystemExit("missing requirements data segment")
payload_escaped = match.group(1)
payload = codecs.decode(payload_escaped, "unicode_escape")
parsed = json.loads(payload)
print(json.dumps(parsed))
PY
  jq -S '.' "${requirements_expected}" > "${TMP_DIR}/${pack}.requirements.expected.json"
  jq -S '.' "${requirements_actual}" > "${TMP_DIR}/${pack}.requirements.actual.json"
  if ! diff -u "${TMP_DIR}/${pack}.requirements.expected.json" "${TMP_DIR}/${pack}.requirements.actual.json"; then
    echo "Requirements output mismatch for ${pack}" >&2
    exit 1
  fi
  if [ ! -f "${answers}" ]; then
    echo "Missing setup.input.json for ${pack}" >&2
    exit 1
  fi
  if [ ! -f "${expected}" ]; then
    echo "Missing setup.expected.plan.json for ${pack}" >&2
    exit 1
  fi

  output_path="${TMP_DIR}/${pack}.json"
  provider_id="$(pack_id_for "${pack}")"
  if [ "$(requires_base_url_for "${pack}")" = "true" ]; then
    greentic-provision dry-run setup \
      --pack "${pack_dir}" \
      --provider-id "${provider_id}" \
      --install-id "${provider_id}-fixture" \
      --public-base-url "https://example.invalid" \
      --answers "${answers}" \
      --json > "${output_path}"
  else
    greentic-provision dry-run setup \
      --pack "${pack_dir}" \
      --provider-id "${provider_id}" \
      --install-id "${provider_id}-fixture" \
      --answers "${answers}" \
      --json > "${output_path}"
  fi

  actual_path="${TMP_DIR}/${pack}.plan.json"
  expected_path="${TMP_DIR}/${pack}.expected.json"
  subscriptions_expected="${fixtures_dir}/subscriptions.expected.json"

  jq -S '.plan' "${output_path}" > "${actual_path}"
  jq -S '.' "${expected}" > "${expected_path}"

  if ! diff -u "${expected_path}" "${actual_path}"; then
    echo "Provisioning plan mismatch for ${pack}" >&2
    exit 1
  fi
  if [ -f "${subscriptions_expected}" ]; then
    jq -S '.plan.subscription_ops' "${output_path}" > "${TMP_DIR}/${pack}.subscriptions.actual.json"
    jq -S '.' "${subscriptions_expected}" > "${TMP_DIR}/${pack}.subscriptions.expected.json"
    if ! diff -u "${TMP_DIR}/${pack}.subscriptions.expected.json" "${TMP_DIR}/${pack}.subscriptions.actual.json"; then
      echo "Subscriptions mismatch for ${pack}" >&2
      exit 1
    fi
  fi

done

echo "Provisioning conformance checks passed."
