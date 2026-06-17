#!/usr/bin/env bash
set -euo pipefail

# Manual deploy script for Almanach Orchestrator.
# Assumes the code has already been pulled to /opt/almanach/app.
# Run this on the VPS after `git pull origin main`.

cd /opt/almanach/app

# Read PORT from .env safely (do NOT source the full file)
PORT=3001
if [[ -f /opt/almanach/.env ]]; then
  ENV_PORT=$(awk -F= '/^[[:space:]]*PORT[[:space:]]*=/ {
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", $2);
    gsub(/^["'\''"]+|["'\''"]+$/, "", $2);
    print $2;
    exit
  }' /opt/almanach/.env)
  if [[ -n "${ENV_PORT}" ]]; then
    PORT="${ENV_PORT}"
  fi
fi

if ! [[ "${PORT}" =~ ^[0-9]+$ ]] || [[ "${PORT}" -lt 1 ]] || [[ "${PORT}" -gt 65535 ]]; then
  echo "ERROR: Invalid PORT value '${PORT}'" >&2
  exit 1
fi

# Stop the running orchestrator via systemd
echo "Stopping current orchestrator..."
systemctl stop almanach-orchestrator

# Build release binary
echo "Building release binary..."
cargo build --release -p almanach-orchestrator

# Swap binary
cp target/release/almanach-orchestrator /opt/almanach/bin/almanach-orchestrator

# Start new binary
echo "Starting new orchestrator..."
systemctl start almanach-orchestrator

# Health check
HEALTH_URL="http://localhost:${PORT}/health"
MAX_RETRIES=10
RETRY_DELAY=2
CURL_OPTS=(-fsS --connect-timeout 3 --max-time 10)

echo "Health checking ${HEALTH_URL}..."
for i in $(seq 1 "${MAX_RETRIES}"); do
  if curl "${CURL_OPTS[@]}" "${HEALTH_URL}" >/dev/null 2>&1; then
    echo "Health check passed!"
    systemctl status almanach-orchestrator --no-pager
    exit 0
  fi
  echo "Attempt $i/${MAX_RETRIES} failed. Retrying in ${RETRY_DELAY}s..."
  sleep "${RETRY_DELAY}"
done

echo "ERROR: Health check failed. Check logs: journalctl -u almanach-orchestrator -n 50"
exit 1
