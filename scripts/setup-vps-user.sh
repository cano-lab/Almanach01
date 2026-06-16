#!/usr/bin/env bash
set -euo pipefail

BASE_DIR="${HOME}/almanach"
APP_DIR="${BASE_DIR}/app"
BIN_DIR="${BASE_DIR}/bin"
SCRIPTS_DIR="${BASE_DIR}/scripts"

echo "=== Almanach user setup ==="

# Create directories
mkdir -p "${APP_DIR}" "${BIN_DIR}" "${SCRIPTS_DIR}"

# Install Rust if needed
if ! command -v cargo &>/dev/null; then
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env"
fi

# Clone repo if app directory is empty
if [ -z "$(ls -A "${APP_DIR}" 2>/dev/null)" ]; then
    echo "Cloning Almanach01 repo into ${APP_DIR}..."
    git clone https://github.com/cano-lab/Almanach01.git "${APP_DIR}"
fi

# Copy deploy script to the scripts directory so GitHub Actions can invoke it
DEPLOY_SCRIPT_SRC="${APP_DIR}/scripts/deploy-user.sh"
DEPLOY_SCRIPT_DST="${SCRIPTS_DIR}/deploy-user.sh"
if [ -f "${DEPLOY_SCRIPT_SRC}" ]; then
    cp "${DEPLOY_SCRIPT_SRC}" "${DEPLOY_SCRIPT_DST}"
    chmod +x "${DEPLOY_SCRIPT_DST}"
    echo "Copied deploy script to ${DEPLOY_SCRIPT_DST}"
else
    echo "WARNING: deploy script not found at ${DEPLOY_SCRIPT_SRC}"
fi

# If user already placed .env at BASE_DIR, symlink it into the app directory
# so the orchestrator's dotenv loader finds it in its CWD
if [ -f "${BASE_DIR}/.env" ] && [ ! -e "${APP_DIR}/.env" ]; then
    ln -s "${BASE_DIR}/.env" "${APP_DIR}/.env"
    echo "Symlinked ${BASE_DIR}/.env to ${APP_DIR}/.env"
fi

echo "=== Setup complete ==="
echo "Next steps:"
echo "  1. Copy your .env file to ${BASE_DIR}/.env (or directly to ${APP_DIR}/.env)"
echo "  2. Copy/seed data to ${APP_DIR}/data/"
echo "  3. Add the GitHub Actions SSH public key to ~/.ssh/authorized_keys"
echo "  4. Run ${DEPLOY_SCRIPT_DST} to build and start the app"
