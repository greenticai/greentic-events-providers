#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"
RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-1.91.0}"
export RUSTUP_TOOLCHAIN

echo "==> cargo fmt"
cargo fmt --all -- --check

echo "==> cargo clippy (native crates only)"
cargo clippy -p provider-common -p provider-core -p provider-webhook -p provider-email -p provider-sms -p provider-timer -p sbom-patch --all-targets -- -D warnings

echo "==> cargo clippy (wasm components)"
cargo clippy --target wasm32-wasip2 -p events-provider-dummy -p events-provider-webhook -p events-provider-timer -p events-provider-sms -p events-provider-email -p events-provider-email-sendgrid -p events-provider-sms-twilio -p stub-component-v060 -- -D warnings

echo "==> cargo test (native crates only)"
cargo test -p provider-common -p provider-core -p provider-webhook -p provider-email -p provider-sms -p provider-timer -p sbom-patch

echo "==> build packs"
bash scripts/build_packs.sh

echo "==> greentic-pack doctor"
mkdir -p dist/packs
packs=(dist/events-*.gtpack)
if [ "${#packs[@]}" -eq 0 ]; then
  echo "No pack artifacts found for doctor validation." >&2
  exit 1
fi
for pack in "${packs[@]}"; do
  cp "${pack}" "dist/packs/$(basename "${pack}")"
done
if ls dist/packs/events-*.gtpack >/dev/null 2>&1; then
  for pack in dist/packs/events-*.gtpack; do
    greentic-pack doctor --validate --pack "${pack}"
  done
else
  echo "No pack artifacts found for doctor validation." >&2
  exit 1
fi

echo "==> greentic-provision conformance"
bash scripts/provision_conformance.sh

echo "==> no hand-rolling CI checks"
bash ci/no_hand_rolling.sh

echo "All checks passed."
