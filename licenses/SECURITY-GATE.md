# NexVigilant Pre-Publish Security Gate
# Version 1.0 — Run before ANY public release

## Mandatory Checks (ALL must pass)

### 1. Secrets Scan
```bash
# Gitleaks on source (exclude target/)
cat > /tmp/gitleaks-nexcore.toml << 'EOF'
[allowlist]
paths = ["target/", ".git/"]
EOF
gitleaks detect --source ~/Projects/Active/nexcore --no-git -c /tmp/gitleaks-nexcore.toml
gitleaks detect --source ~/Projects/Active/nucleus --no-git -c /tmp/gitleaks-nexcore.toml
gitleaks detect --source ~/Projects/rsk-core --no-git -c /tmp/gitleaks-nexcore.toml
```
**Gate:** Zero leaks found on all three repos.

### 2. Hardcoded Key Scan
```bash
grep -rn "api_key\s*=\s*\"[a-zA-Z0-9_-]\{10,\}\"" \
  ~/Projects/Active/nexcore/crates/ \
  --include="*.rs" \
  --exclude-dir=target
```
**Gate:** Zero matches. All API keys must come from env vars.

### 3. Env File Audit
```bash
find ~/Projects/Active/nexcore ~/Projects/Active/nucleus ~/Projects/rsk-core \
  -name ".env" -o -name ".env.local" -o -name ".env.production" | \
  grep -v node_modules | grep -v target
```
**Gate:** No `.env` files with real secrets. Only `.env.example` templates.

### 4. Config File Scan
```bash
# Ensure no secrets in publishable configs
grep -rn "Bearer\|sk-\|gho_\|ghp_\|whub_\|firebase.*apiKey" \
  ~/Projects/Active/nexcore/webmcp-configs/ 2>/dev/null
```
**Gate:** Zero matches in any config destined for WebMCP Hub.

### 5. License Header Check
```bash
# Every publishable config must have the license header
for f in ~/Projects/Active/nexcore/webmcp-configs/*.json; do
  head -3 "$f" | grep -q "NexVigilant" || echo "MISSING HEADER: $f"
done
```
**Gate:** Every config file contains the NexVigilant license header.

### 6. Dependency Audit
```bash
cd ~/Projects/Active/nexcore && cargo audit
cd ~/Projects/Active/nucleus && npm audit --omit=dev
```
**Gate:** Zero critical vulnerabilities.

### 7. Domain Allowlist Integrity
```bash
# Verify the email domain allowlist hasn't been tampered with
md5sum ~/Projects/Active/nexcore/licenses/ALLOWED-DOMAINS.md
```
**Gate:** Checksum matches the last reviewed version.

## Never Publish

These files MUST NEVER appear in any public config or marketplace:

| File | Contains |
|------|----------|
| `~/.claude.json` | All MCP server API keys |
| `~/.env` | System-level secrets |
| `nucleus/.env.local` | Firebase config |
| Any `*.pem`, `*.key`, `*.p12` | Certificates |
| `brain.db` | Session data, telemetry |

## Pre-Publish Checklist

- [ ] Gitleaks passes on all 3 repos
- [ ] No hardcoded API keys in source
- [ ] No .env files with real secrets
- [ ] No secrets in WebMCP config files
- [ ] License header on every config
- [ ] Dependency audit clean
- [ ] Domain allowlist checksum verified
- [ ] Config reviewed by Matthew before publish
- [ ] Version number incremented
- [ ] Changelog updated
