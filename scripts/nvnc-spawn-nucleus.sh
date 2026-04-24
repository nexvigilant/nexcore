#!/usr/bin/env bash
# NVNC Nucleus spawn helper.
#
# Reads scripts/nvnc-nucleus.unit.toml, converts the [spec] block into
# a `systemd-run --user` invocation that LiveSystemdRuntime would issue,
# and executes it directly. This is a shell-script stand-in for the
# future `nvnc-migrate` CLI.
#
# Preconditions:
#   - systemd-user session available (systemctl --user works)
#   - Bundle rsynced to /opt/nvnc/nucleus/ (run `just nvnc-build-nucleus`)
#   - Node installed at /usr/bin/node
#
# Usage:
#   bash scripts/nvnc-spawn-nucleus.sh
#   # Override bundle path for debugging against the in-place nexdev build:
#   NVNC_BUNDLE=~/Projects/Active/nucleus/app/.next/standalone bash scripts/nvnc-spawn-nucleus.sh

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
spec_file="$repo_root/scripts/nvnc-nucleus.unit.toml"

if [[ ! -f "$spec_file" ]]; then
  echo "✗ spec not found: $spec_file" >&2
  exit 1
fi

# Minimal TOML field extraction via python3 (already a sovereign path —
# python ships with the OS; no new external dep).
python_extract() {
  python3 - "$spec_file" <<'PY' "$@"
import sys, tomllib
with open(sys.argv[1], 'rb') as f:
    d = tomllib.load(f)
print(d['spec']['name'])
print(d['spec']['image'])
for a in d['spec']['args']:
    print(a)
print('---env---')
for k, v in d.get('environment', {}).items():
    print(f"{k}={v}")
PY
}

mapfile -t lines < <(python_extract)

spec_name="${lines[0]}"
spec_image="${lines[1]}"

# Collect args until the env separator
spec_args=()
env_pairs=()
in_env=false
for l in "${lines[@]:2}"; do
  if [[ "$l" == "---env---" ]]; then
    in_env=true
    continue
  fi
  if $in_env; then
    env_pairs+=("--setenv=$l")
  else
    spec_args+=("$l")
  fi
done

# Override bundle path if NVNC_BUNDLE set (useful for in-place nexdev testing)
if [[ -n "${NVNC_BUNDLE:-}" ]]; then
  spec_args=("${NVNC_BUNDLE}/server.js")
  echo "[spawn] using NVNC_BUNDLE override: ${spec_args[0]}"
fi

unit="nvnc-${spec_name}.service"

echo "[spawn] unit:  $unit"
echo "[spawn] image: $spec_image"
echo "[spawn] args:  ${spec_args[*]}"
echo "[spawn] env:   ${env_pairs[*]}"
echo

# Execute the systemd-run invocation that LiveSystemdRuntime would issue.
systemd-run --user \
  --unit="$unit" \
  "${env_pairs[@]}" \
  "$spec_image" "${spec_args[@]}"

echo
echo "[spawn] ✓ spawned $unit"
echo "[spawn] observe:  systemctl --user status $unit"
echo "[spawn] logs:     journalctl --user -u $unit --follow"
