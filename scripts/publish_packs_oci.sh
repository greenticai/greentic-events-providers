#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
REGISTRY="${REGISTRY:-ghcr.io}"
OWNER="${OWNER:-greenticai}"
REPO="${REPO:-greentic-packs}"
SOURCE_ANNOTATION="https://github.com/greenticai/greentic-events-providers"
GITHUB_SHA="${GITHUB_SHA:-$(git -C "${ROOT_DIR}" rev-parse --verify HEAD)}"
MAKE_PUBLIC="${MAKE_PUBLIC:-false}"
GHCR_TOKEN="${GHCR_TOKEN:-${GITHUB_TOKEN:-}}"

determine_version() {
  if [ -n "${VERSION:-}" ]; then
    echo "${VERSION}"
    return
  fi

  if tag="$(git -C "${ROOT_DIR}" describe --tags --exact-match 2>/dev/null)"; then
    echo "${tag#v}"
    return
  fi

  version_from_python="$(
    python3 - <<'PY' 2>/dev/null || true
import importlib
import pathlib

try:
    toml = importlib.import_module("tomllib")
except ModuleNotFoundError:
    try:
        toml = importlib.import_module("tomli")
    except ModuleNotFoundError:
        raise SystemExit

root = pathlib.Path(__file__).resolve().parent.parent
data = toml.loads((root / "Cargo.toml").read_text())
print(data.get("workspace", {}).get("package", {}).get("version", ""))
PY
  )"
  if [ -n "${version_from_python}" ]; then
    echo "${version_from_python}"
    return
  fi

  version_from_awk="$(awk '
    $0 ~ /^\[workspace\.package\]/ { in_section=1; next }
    in_section && $0 ~ /^\[/ { in_section=0 }
    in_section && $1 ~ /^version/ {
      gsub(/"/, "", $3);
      print $3;
      exit
    }
  ' "${ROOT_DIR}/Cargo.toml")"
  if [ -n "${version_from_awk}" ]; then
    echo "${version_from_awk}"
    return
  fi
}

VERSION_RESOLVED="$(determine_version)"
if [ -z "${VERSION_RESOLVED}" ]; then
  echo "VERSION not provided and could not determine from git tag or Cargo.toml" >&2
  exit 1
fi

if [ ! -d "${DIST_DIR}" ]; then
  echo "Dist directory ${DIST_DIR} not found. Run scripts/build_packs.sh first." >&2
  exit 1
fi

shopt -s nullglob
PACKS=("${DIST_DIR}"/events-*.gtpack)
if [ "${#PACKS[@]}" -eq 0 ]; then
  echo "No pack artifacts found under ${DIST_DIR}" >&2
  exit 1
fi

for pack in "${PACKS[@]}"; do
  pack_name="$(basename "${pack%.gtpack}")"
  ref="${REGISTRY}/${OWNER}/${REPO}/${pack_name}:${VERSION_RESOLVED}"

  echo "Pushing ${pack_name} -> ${ref}"
  (
    cd "${DIST_DIR}"
    oras push "${ref}" \
    "$(basename "${pack}"):application/vnd.greentic.gtpack+zip" \
    --annotation org.opencontainers.image.source="${SOURCE_ANNOTATION}" \
    --annotation org.opencontainers.image.revision="${GITHUB_SHA}" \
    --annotation org.opencontainers.image.version="${VERSION_RESOLVED}" \
    --annotation org.opencontainers.image.title="${pack_name}"
  )

  digest=""
  if command -v jq >/dev/null 2>&1 && oras manifest fetch --help 2>&1 | grep -q -- "--descriptor"; then
    digest="$(oras manifest fetch --descriptor "${ref}" 2>/dev/null | jq -r '.digest // .Descriptor.digest // empty' || true)"
  fi

  if [ -z "${digest}" ] && command -v sha256sum >/dev/null 2>&1; then
    digest="$(oras manifest fetch "${ref}" 2>/dev/null | sha256sum | awk '{print "sha256:"$1}' || true)"
  fi

  if [ -n "${digest}" ]; then
    echo "Digest for ${ref}: ${digest}"
  else
    echo "Digest for ${ref}: (unavailable - oras digest lookup failed)" >&2
  fi

  if [ "${MAKE_PUBLIC}" = "true" ] && [ -n "${GHCR_TOKEN}" ]; then
    PKG="${REPO}/${pack_name}"
    PKG_ENC="$(
      PKG="${PKG}" python3 - <<'PY' 2>/dev/null || true
import os
import urllib.parse

print(urllib.parse.quote(os.environ["PKG"], safe=""))
PY
    )"
    if [ -n "${PKG_ENC}" ]; then
      VIS_URL="https://api.github.com/orgs/${OWNER}/packages/container/${PKG_ENC}/visibility"
      echo "Setting visibility public for ${OWNER}/${PKG_ENC}"
      echo "Visibility URL: ${VIS_URL}"
      if ! curl -sS -X PATCH \
        -H "Authorization: Bearer ${GHCR_TOKEN}" \
        -H "Accept: application/vnd.github+json" \
        -H "Content-Type: application/json" \
        "${VIS_URL}" \
        -d '{"visibility":"public"}'; then
        echo "Visibility update failed (non-fatal)." >&2
      else
        echo
      fi
    else
      echo "Visibility update skipped (failed to encode package name)." >&2
    fi
  fi
done
