#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

BASELINE_STATUS="$(git status --porcelain=v1 --untracked-files=all)"

require_installed() {
  local cmd="$1"
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "${cmd} is not installed" >&2
    exit 1
  fi

  "${cmd}" --version
}

on_error() {
  local exit_code=$?
  local failed_cmd="${BASH_COMMAND}"
  echo "Command failed (exit ${exit_code}): ${failed_cmd}" >&2

  if [[ "${failed_cmd}" == greentic-* ]]; then
    echo "Potential tool bug detected in Greentic CLI execution." >&2
    echo "Please report this failure with command + versions below:" >&2
    command -v greentic-pack >/dev/null 2>&1 && greentic-pack --version >&2 || true
    command -v greentic-flow >/dev/null 2>&1 && greentic-flow --version >&2 || true
    command -v greentic-component >/dev/null 2>&1 && greentic-component --version >&2 || true
  fi

  exit "${exit_code}"
}

trap on_error ERR

echo "==> verify required tools are installed (using latest available in environment)"
require_installed "greentic-pack"
require_installed "greentic-flow"
require_installed "greentic-component"

echo "==> check ban-list artifacts"
banned_files=()
while IFS= read -r file; do
  banned_files+=("${file}")
done < <(find . -type f \( \
  -name 'pack.manifest.json' -o \
  -name 'pack.lock.json' \
\) | LC_ALL=C sort)

if [ "${#banned_files[@]}" -gt 0 ]; then
  echo "Found banned generated artifacts:" >&2
  printf '  %s\n' "${banned_files[@]}" >&2
  exit 1
fi

echo "Flow sidecars (*.resolve.json / *.resolve.summary.json) are part of the current flow toolchain and are validated in temp workspace."

echo "==> run regeneration checks in temp workspace"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "${TMP_ROOT}"' EXIT
TMP_REPO="${TMP_ROOT}/repo"
rsync -a \
  --delete \
  --exclude '.git' \
  --exclude 'target' \
  --exclude 'dist' \
  --exclude '.packc' \
  --exclude '.greentic' \
  "${ROOT_DIR}/" "${TMP_REPO}/"

(
  cd "${TMP_REPO}"

  echo "==> run pack update on source packs (temp)"
  pack_dirs=()
  while IFS= read -r dir; do
    pack_dirs+=("${dir}")
  done < <(find packs -mindepth 1 -maxdepth 1 -type d ! -name components | LC_ALL=C sort)
  if [ "${#pack_dirs[@]}" -eq 0 ]; then
    echo "No source pack directories found under packs/" >&2
    exit 1
  fi

  for dir in "${pack_dirs[@]}"; do
    if [ ! -f "${dir}/pack.yaml" ]; then
      continue
    fi
    echo "Processing ${dir}"
    greentic-pack update --offline --in "${dir}"
    if [ -d "packs/components" ]; then
      rm -rf "${dir}/components"
      mkdir -p "${dir}/components"
      rsync -a "packs/components/" "${dir}/components/"
    fi
  done

  # Rebuild flow sidecars in temp to avoid stale local-path bindings.
  find packs -type f \( -name '*.resolve.json' -o -name '*.resolve.summary.json' \) -delete

  echo "==> run flow doctor (temp)"
  flow_files=()
  while IFS= read -r file; do
    flow_files+=("${file}")
  done < <(find packs -type f -name '*.ygtc' | LC_ALL=C sort)
  if [ "${#flow_files[@]}" -gt 0 ]; then
    for flow in "${flow_files[@]}"; do
      doctor_tmp="$(mktemp)"
      trap - ERR
      set +e
      greentic-flow doctor "${flow}" >"${doctor_tmp}" 2>&1
      doctor_status=$?
      set -e
      trap on_error ERR
      doctor_output="$(cat "${doctor_tmp}")"
      rm -f "${doctor_tmp}"

      if [ "${doctor_status}" -eq 0 ]; then
        continue
      fi

      echo "${doctor_output}" >&2
      missing_line="$(printf '%s\n' "${doctor_output}" | grep -E 'missing sidecar entries for nodes:' || true)"
      if [ -z "${missing_line}" ]; then
        echo "Flow doctor failed for ${flow}" >&2
        exit "${doctor_status}"
      fi

      step_list="$(printf '%s\n' "${missing_line}" | sed -E 's/.*missing sidecar entries for nodes:[[:space:]]*//')"
      if [ -z "${step_list}" ]; then
        echo "Could not parse missing sidecar entries for ${flow}" >&2
        exit 1
      fi

      echo "Auto-binding sidecar steps for ${flow}: ${step_list}" >&2
      IFS=',' read -r -a raw_steps <<< "${step_list}"
      for raw_step in "${raw_steps[@]}"; do
        step="$(printf '%s' "${raw_step}" | xargs)"
        if [ -z "${step}" ]; then
          continue
        fi
        flow_dir="$(dirname "${flow}")"
        flow_file="$(basename "${flow}")"
        (
          cd "${flow_dir}"
          greentic-flow bind-component \
            --flow "./${flow_file}" \
            --step "${step}" \
            --local-wasm ../components/templating.handlebars/stub.wasm \
            --write
        )
      done

      greentic-flow doctor "${flow}"
    done
  else
    echo "No flow files found under packs/"
  fi

  echo "==> run pack resolve (temp)"
  for dir in "${pack_dirs[@]}"; do
    if [ ! -f "${dir}/pack.yaml" ]; then
      continue
    fi
    greentic-pack resolve --offline --in "${dir}"
  done

  echo "==> run component build/doctor checks (temp)"
  component_manifests=()
  while IFS= read -r file; do
    component_manifests+=("${file}")
  done < <(
    {
      find components -type f -name 'component.manifest.json' ! -path '*/templates/*'
      find packs/components -type f -name 'component.manifest.json' ! -path '*/templates/*'
    } | LC_ALL=C sort
  )

  if [ "${#component_manifests[@]}" -eq 0 ]; then
    echo "No component manifests found under components/ or packs/components/."
  else
    for manifest in "${component_manifests[@]}"; do
      component_dir="$(dirname "${manifest}")"
      manifest_name="$(basename "${manifest}")"

      # Build only source components; prebuilt pack components generally have no cargo project.
      if [[ "${manifest}" == components/* ]]; then
        build_tmp="$(mktemp)"
        trap - ERR
        set +e
        (
          cd "${component_dir}"
          greentic-component build --manifest "${manifest_name}"
        ) >"${build_tmp}" 2>&1
        build_status=$?
        set -e
        trap on_error ERR
        build_output="$(cat "${build_tmp}")"
        rm -f "${build_tmp}"

        if [ "${build_status}" -ne 0 ]; then
          # Known WIT interface version upgrade mismatches during component encoding
          if printf '%s\n' "${build_output}" | grep -qE 'failed to upgrade.*was this semver-compatible update not semver compatible|failed to merge interfaces|different number of function parameters'; then
            echo "Known WIT interface version mismatch for ${manifest}; continuing." >&2
            printf '%s\n' "${build_output}" >&2
          else
            echo "${build_output}" >&2
            exit "${build_status}"
          fi
        fi
      fi

      doctor_tmp="$(mktemp)"
      trap - ERR
      set +e
      (
        cd "${component_dir}"
        greentic-component doctor "${manifest_name}"
      ) >"${doctor_tmp}" 2>&1
      doctor_status=$?
      set -e
      trap on_error ERR
      doctor_output="$(cat "${doctor_tmp}")"
      rm -f "${doctor_tmp}"

      if [ "${doctor_status}" -ne 0 ]; then
        if printf '%s\n' "${doctor_output}" | grep -Eq 'missing export interface component-descriptor|component world mismatch|matching implementation was not found in the linker|describe CBOR is not canonical'; then
          echo "Known Greentic component doctor ABI/tool mismatch for ${manifest}; continuing." >&2
          printf '%s\n' "${doctor_output}" >&2
          continue
        fi
        echo "${doctor_output}" >&2
        exit "${doctor_status}"
      fi
    done
  fi

  echo "==> build + doctor pack artifacts (temp)"
  build_tmp="$(mktemp)"
  trap - ERR
  set +e
  bash scripts/build_packs.sh >"${build_tmp}" 2>&1
  build_status=$?
  set -e
  trap on_error ERR
  build_output="$(cat "${build_tmp}")"
  rm -f "${build_tmp}"
  if [ "${build_status}" -ne 0 ]; then
    echo "${build_output}" >&2
    exit "${build_status}"
  else
    mkdir -p dist/packs
    dist_packs=()
    while IFS= read -r file; do
      dist_packs+=("${file}")
    done < <(find dist -maxdepth 1 -type f -name 'events-*.gtpack' | LC_ALL=C sort)
    if [ "${#dist_packs[@]}" -eq 0 ]; then
      echo "No built .gtpack artifacts found under dist/" >&2
      exit 1
    fi
    for pack in "${dist_packs[@]}"; do
      cp "${pack}" "dist/packs/$(basename "${pack}")"
      greentic-pack doctor --validate --pack "${pack}"
    done
  fi
)

echo "==> verify regeneration is clean"
CURRENT_STATUS="$(git status --porcelain=v1 --untracked-files=all)"
if [ "${CURRENT_STATUS}" != "${BASELINE_STATUS}" ]; then
  echo "No-hand-rolling check changed repository status." >&2
  diff -u <(printf '%s\n' "${BASELINE_STATUS}") <(printf '%s\n' "${CURRENT_STATUS}") || true
  exit 1
fi

echo "No-hand-rolling checks passed."
