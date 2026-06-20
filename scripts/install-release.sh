#!/usr/bin/env bash
# Install a published almanach-orchestrator release binary on the VPS.
#
# Usage:
#   ./scripts/install-release.sh                 # installs the "latest" release
#   ./scripts/install-release.sh v0.1.0          # installs a specific tag
#
# Workflow on the VPS:
#   cd /opt/almanach/app
#   git pull
#   ./scripts/install-release.sh
#
# What it does:
#   1. Downloads `almanach-orchestrator` from the requested release.
#   2. Verifies the file is a valid ELF binary.
#   3. Backs up the current `/opt/almanach/bin/almanach-orchestrator` to
#      `…/almanach-orchestrator.prev`.
#   4. Installs the new binary in place (atomic via `install`).
#   5. Restarts the `almanach` systemd unit and verifies it stays up.
#   6. On failure to start, restores the previous binary.

set -euo pipefail

TAG="${1:-latest}"
REPO="cano-lab/Almanach01"
BIN_NAME="almanach-orchestrator"
INSTALL_DIR="/opt/almanach/bin"
INSTALL_PATH="${INSTALL_DIR}/${BIN_NAME}"
BACKUP_PATH="${INSTALL_PATH}.prev"
SERVICE="almanach"

if [[ "$TAG" == "latest" ]]; then
    URL="https://github.com/${REPO}/releases/latest/download/${BIN_NAME}"
else
    URL="https://github.com/${REPO}/releases/download/${TAG}/${BIN_NAME}"
fi

echo "==> Downloading ${TAG} from ${URL}"
TMP="$(mktemp)"
trap 'rm -f "$TMP"' EXIT

# `-f` so curl exits non-zero on HTTP errors (so we don't install a 404 page).
curl -fSL --progress-bar -o "$TMP" "$URL"

# Sanity check: must be an ELF executable, not an HTML error page.
if ! file "$TMP" | grep -q "ELF.*executable"; then
    echo "ERROR: downloaded artifact is not an ELF executable" >&2
    file "$TMP" >&2
    exit 1
fi
DOWNLOAD_SHA="$(sha256sum "$TMP" | awk '{print $1}')"
DOWNLOAD_SIZE="$(stat -c %s "$TMP")"
echo "    size:   ${DOWNLOAD_SIZE} bytes"
echo "    sha256: ${DOWNLOAD_SHA}"

# Make sure the install directory exists.
sudo install -d -m755 "$INSTALL_DIR"

# Back up the current binary so we can roll back if the new one won't start.
if [[ -f "$INSTALL_PATH" ]]; then
    sudo cp -p "$INSTALL_PATH" "$BACKUP_PATH"
    echo "==> Previous binary backed up to ${BACKUP_PATH}"
fi

echo "==> Installing to ${INSTALL_PATH}"
sudo install -m755 "$TMP" "$INSTALL_PATH"

echo "==> Restarting ${SERVICE}"
sudo systemctl restart "$SERVICE"

# Give the service a moment to either come up cleanly or crash.
sleep 2

if sudo systemctl is-active --quiet "$SERVICE"; then
    echo "==> ${SERVICE} is active. Done."
    # Show a short status block so the operator sees recent log lines.
    sudo systemctl status "$SERVICE" --no-pager -n 5 || true
    exit 0
fi

echo "ERROR: ${SERVICE} failed to start. Rolling back to previous binary." >&2
if [[ -f "$BACKUP_PATH" ]]; then
    sudo install -m755 "$BACKUP_PATH" "$INSTALL_PATH"
    sudo systemctl restart "$SERVICE"
    if sudo systemctl is-active --quiet "$SERVICE"; then
        echo "    Rollback succeeded; ${SERVICE} is active again on the previous binary." >&2
    else
        echo "    Rollback ALSO failed. The service is down. Investigate with:" >&2
        echo "        sudo systemctl status ${SERVICE}" >&2
        echo "        sudo journalctl -u ${SERVICE} -n 50" >&2
    fi
else
    echo "    No backup available to roll back to." >&2
fi
exit 1
