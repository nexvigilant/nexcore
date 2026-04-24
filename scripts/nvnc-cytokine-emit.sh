#!/usr/bin/env bash
# nvnc-cytokine-emit — send a cytokine_emit call to nexcore-mcp via stdio.
#
# Usage:
#   nvnc-cytokine-emit <family> <name> <severity> <scope> <payload_json>
#
# Returns 0 on success (tools/call produced a result), 1 on failure.
# nexcore-mcp HTTP REST blocks cytokine_emit (mutating bus ops are not
# reachable via the public allowlist), so this script uses the trusted
# stdio transport that Claude Code itself uses.
set -euo pipefail

MCP_BIN="${NEXCORE_MCP_BIN:-/home/matthew/Projects/Active/nucleus/workspaces/nexcore/target/release/nexcore-mcp}"
TIMEOUT="${NVNC_CYTOKINE_TIMEOUT:-5}"

family="${1:?family required}"
name="${2:?name required}"
severity="${3:-low}"
scope="${4:-systemic}"
payload="${5:-{\}}"

if [[ ! -x "$MCP_BIN" ]]; then
  echo "nvnc-cytokine-emit: mcp binary missing at $MCP_BIN" >&2
  exit 2
fi

# JSON-RPC 2.0 over stdio:
#   1. initialize
#   2. notifications/initialized
#   3. tools/call cytokine_emit
# The sleep between 2 and 3 lets nexcore-mcp finish processing the
# handshake before the tool call — without it, id=2 response is lost.
resp=$(
  {
    echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"nvnc-cytokine-emit","version":"1"}}}'
    echo '{"jsonrpc":"2.0","method":"notifications/initialized"}'
    sleep 0.3
    printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"cytokine_emit","arguments":{"family":"%s","name":"%s","severity":"%s","scope":"%s","payload":%s}}}\n' \
      "$family" "$name" "$severity" "$scope" "$payload"
    sleep 0.3
  } | timeout "$TIMEOUT" "$MCP_BIN" 2>/dev/null | grep -F '"id":2' | head -1
)

if [[ -z "$resp" ]]; then
  echo "nvnc-cytokine-emit: no response from mcp" >&2
  exit 1
fi

if grep -q '"error"' <<<"$resp"; then
  echo "nvnc-cytokine-emit: mcp error: $resp" >&2
  exit 1
fi

echo "$resp"
exit 0
