#!/usr/bin/env bash
set -euo pipefail

BASE_DIR="${HOME}/almanach"
APP_DIR="${BASE_DIR}/app"
BIN_DIR="${BASE_DIR}/bin"
PID_FILE="${BASE_DIR}/almanach.pid"
LOG_FILE="${BASE_DIR}/almanach.log"
ENV_FILE="${BASE_DIR}/.env"
BINARY="${BIN_DIR}/almanach-orchestrator"
BACKUP_BINARY="${BIN_DIR}/almanach-orchestrator.backup"

echo "=== Almanach user deploy ==="

cd "${APP_DIR}"

# Update code
echo "Fetching latest code..."
git fetch origin main
git reset --hard origin/main

# Build release binary
echo "Building release binary..."
cargo build --release -p almanach-orchestrator

# Read PORT safely from env file (do not source full file)
PORT=3001
if [ -f "${ENV_FILE}" ]; then
    # Extract first valid PORT= line, ignore comments
    RAW_PORT=$(grep -E '^\s*PORT\s*=\s*[0-9]+' "${ENV_FILE}" | head -n1 | sed -E 's/.*=\s*([0-9]+).*/\1/') || true
    if [ -n "${RAW_PORT}" ]; then
        if [ "${RAW_PORT}" -ge 1 ] && [ "${RAW_PORT}" -le 65535 ]; then
            PORT="${RAW_PORT}"
        else
            echo "Warning: PORT in ${ENV_FILE} is out of range (1-65535), using default ${PORT}"
        fi
    fi
fi

echo "Using PORT=${PORT}"

# Stop existing process gracefully
if [ -f "${PID_FILE}" ]; then
    OLD_PID=$(cat "${PID_FILE}")
    if kill -0 "${OLD_PID}" 2>/dev/null; then
        echo "Stopping existing process ${OLD_PID}..."
        kill "${OLD_PID}" || true
        for _ in {1..10}; do
            if ! kill -0 "${OLD_PID}" 2>/dev/null; then
                break
            fi
            sleep 1
        done
        if kill -0 "${OLD_PID}" 2>/dev/null; then
            echo "Force killing process ${OLD_PID}..."
            kill -9 "${OLD_PID}" || true
        fi
    fi
    rm -f "${PID_FILE}"
fi

# Backup current binary
if [ -f "${BINARY}" ]; then
    cp "${BINARY}" "${BACKUP_BINARY}"
fi

# Copy new binary
cp "${APP_DIR}/target/release/almanach-orchestrator" "${BINARY}"

# Ensure the app can see the env file in its CWD via a symlink
if [ -f "${HOME}/almanach/.env" ] && [ ! -e "${APP_DIR}/.env" ]; then
    ln -s "${HOME}/almanach/.env" "${APP_DIR}/.env"
fi

# Export PORT so the launched binary definitely sees it, even if .env loading is delayed
export PORT

# Start new process with nohup so it survives logout
echo "Starting new process on port ${PORT}..."
nohup "${BINARY}" > "${LOG_FILE}" 2>&1 &
NEW_PID=$!
echo "${NEW_PID}" > "${PID_FILE}"

# Health check
HEALTH_URL="http://localhost:${PORT}/health"
MAX_RETRIES=30
RETRY_DELAY=1

echo "Health checking ${HEALTH_URL}..."
for ((i=1; i<=MAX_RETRIES; i++)); do
    if curl -s --connect-timeout 3 --max-time 10 "${HEALTH_URL}" >/dev/null; then
        echo "Health check passed! Almanach is running on port ${PORT} (PID ${NEW_PID})."
        exit 0
    fi
    sleep "${RETRY_DELAY}"
done

# Health check failed — rollback
echo "Health check failed. Rolling back..."

# Kill new process
if kill -0 "${NEW_PID}" 2>/dev/null; then
    kill "${NEW_PID}" || true
    sleep 2
    if kill -0 "${NEW_PID}" 2>/dev/null; then
        kill -9 "${NEW_PID}" || true
    fi
fi

# Restore backup and start it
if [ -f "${BACKUP_BINARY}" ]; then
    cp "${BACKUP_BINARY}" "${BINARY}"
    nohup "${BINARY}" > "${LOG_FILE}" 2>&1 &
    ROLLBACK_PID=$!
    echo "${ROLLBACK_PID}" > "${PID_FILE}"

    echo "Health checking rollback..."
    for ((i=1; i<=MAX_RETRIES; i++)); do
        if curl -s --connect-timeout 3 --max-time 10 "${HEALTH_URL}" >/dev/null; then
            echo "Rollback succeeded. Almanach is running (PID ${ROLLBACK_PID})."
            exit 1
        fi
        sleep "${RETRY_DELAY}"
    done
    echo "Rollback also failed."
    exit 2
else
    echo "No backup binary available. Rollback impossible."
    exit 2
fi
