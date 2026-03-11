#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/dist"
REGISTRY="${REGISTRY:-ghcr.io}"
OWNER="${OWNER:-greenticai}"
PACKAGE_NAMESPACE="${PACKAGE_NAMESPACE:-packs/events}"
SOURCE_ANNOTATION="https://github.com/greenticai/greentic-events-providers"
GITHUB_SHA="${GITHUB_SHA:-$(git -C "${ROOT_DIR}" rev-parse --verify HEAD)}"
ARTIFACT_TYPE="${ARTIFACT_TYPE:-application/vnd.greentic.gtpack.v1}"
LAYER_MEDIA_TYPE="${LAYER_MEDIA_TYPE:-application/vnd.greentic.gtpack.layer.v1+tar}"

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

build_package_name() {
  local artifact_name="$1"

  if [ -n "${PACKAGE_NAMESPACE}" ]; then
    printf '%s/%s' "${PACKAGE_NAMESPACE}" "${artifact_name}"
  else
    printf '%s' "${artifact_name}"
  fi
}

for pack in "${PACKS[@]}"; do
  pack_name="$(basename "${pack%.gtpack}")"
  artifact_name="${pack_name}.gtpack"
  package_name="$(build_package_name "${pack_name}")"
  ref="${REGISTRY}/${OWNER}/${package_name}:${VERSION_RESOLVED}"
  latest_ref="${REGISTRY}/${OWNER}/${package_name}:latest"

  echo "Pushing ${pack_name} -> ${ref}"
  (
    cd "${DIST_DIR}"
    oras push "${ref}" \
      --artifact-type "${ARTIFACT_TYPE}" \
      --annotation org.opencontainers.image.source="${SOURCE_ANNOTATION}" \
      --annotation org.opencontainers.image.revision="${GITHUB_SHA}" \
      --annotation org.opencontainers.image.version="${VERSION_RESOLVED}" \
      --annotation org.opencontainers.image.title="${artifact_name}" \
      "$(basename "${pack}"):${LAYER_MEDIA_TYPE}"
  )

  echo "Pushing ${pack_name} -> ${latest_ref}"
  (
    cd "${DIST_DIR}"
    oras push "${latest_ref}" \
      --artifact-type "${ARTIFACT_TYPE}" \
      --annotation org.opencontainers.image.source="${SOURCE_ANNOTATION}" \
      --annotation org.opencontainers.image.revision="${GITHUB_SHA}" \
      --annotation org.opencontainers.image.version="${VERSION_RESOLVED}" \
      --annotation org.opencontainers.image.title="${artifact_name}" \
      "$(basename "${pack}"):${LAYER_MEDIA_TYPE}"
  )
done
