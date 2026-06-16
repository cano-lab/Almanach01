#!/usr/bin/env bash
set -euo pipefail

# Deployment script for Almanach Orchestrator
# Runs on the VPS as the deploy user

cd /opt/almanach/app

# Mark this directory as safe for git in case the script is run as a
# different user than the repo owner (e.g. root vs deploy).
git config --global --add safe.directory /opt/almanach/app 2>/dev/null || true

# Pull latest code
echo "Pulling latest code..."
git fetch origin main
git reset --hard origin/main

# Build release binary
echo "Building release binary..."
cargo build --release -p almanach-orchestrator

# Read PORT from .env or default to 3001.
# We intentionally do NOT source the full .env file into this shell,
# because it is meant for the systemd service and may contain values
# that are unsafe or malformed for shell evaluation.
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

# Validate PORT is a positive integer
if ! [[ "${PORT}" =~ ^[0-9]+$ ]] || [[ "${PORT}" -lt 1 ]] || [[ "${PORT}" -gt 65535 ]]; then
  echo "ERROR: Invalid PORT value '${PORT}' from /opt/almanach/.env; must be 1-65535." >&2
  exit 1
fi

# Backup current binary if it exists
if [[ -f /opt/almanach/bin/almanach-orchestrator ]]; then
  echo "Backing up current binary..."
  cp /opt/almanach/bin/almanach-orchestrator /opt/almanach/bin/almanach-orchestrator.backup
fi

# Stop service
echo "Stopping service..."
sudo systemctl stop almanach-orchestrator

# Swap binary
echo "Swapping binary..."
cp target/release/almanach-orchestrator /opt/almanach/bin/almanach-orchestrator

# Start service
echo "Starting service..."
sudo systemctl start almanach-orchestrator

# Health check with retries
echo "Running health check on port ${PORT}..."
HEALTH_URL="http://localhost:${PORT}/health"
MAX_RETRIES=10
RETRY_DELAY=3
CURL_OPTS=(-fsS --connect-timeout 3 --max-time 10)
for i in $(seq 1 "${MAX_RETRIES}"); do
  if curl "${CURL_OPTS[@]}" "${HEALTH_URL}" >/dev/null 2>&1; then
    echo "Health check passed!"
    exit 0
  fi
  echo "Health check attempt $i/${MAX_RETRIES} failed. Retrying in ${RETRY_DELAY}s..."
  sleep "${RETRY_DELAY}"
done

# Health check failed — rollback
echo "ERROR: Health check failed after ${MAX_RETRIES} attempts. Initiating rollback..."

sudo systemctl stop almanach-orchestrator

if [[ -f /opt/almanach/bin/almanach-orchestrator.backup ]]; then
  cp /opt/almanach/bin/almanach-orchestrator.backup /opt/almanach/bin/almanach-orchestrator
  echo "Restored backup binary."
else
  echo "WARNING: No backup binary found to restore."
fi

sudo systemctl start almanach-orchestrator

# Verify rollback health
echo "Checking health after rollback..."
for i in $(seq 1 "${MAX_RETRIES}"); do
  if curl "${CURL_OPTS[@]}" "${HEALTH_URL}" >/dev/null 2>&1; then
    echo "Rollback health check passed. Deployment failed but rollback succeeded."
    exit 1
  fi
  sleep "${RETRY_DELAY}"
done

echo "CRITICAL: Rollback also failed. Manual intervention required."
exit 2
