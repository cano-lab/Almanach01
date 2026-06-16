#!/usr/bin/env bash
set -euo pipefail

# One-time idempotent VPS provisioning script for Almanach
# Run this as root on the VPS

# Determine the repo root so this script works whether run from inside the repo or elsewhere
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Create system user and group
if ! getent group almanach >/dev/null 2>&1; then
  groupadd --system almanach
  echo "Created group almanach"
fi

if ! id -u almanach >/dev/null 2>&1; then
  useradd --system \
    --home-dir /opt/almanach \
    --shell /usr/sbin/nologin \
    --gid almanach \
    almanach
  echo "Created user almanach"
fi

# Ensure the almanach home directory is traversable by the service user
mkdir -p /opt/almanach
chown almanach:almanach /opt/almanach
chmod 755 /opt/almanach

if ! id -u deploy >/dev/null 2>&1; then
  useradd \
    --create-home \
    --home-dir /home/deploy \
    --shell /bin/bash \
    deploy
  echo "Created user deploy"
fi

# Create directories
mkdir -p /opt/almanach/app
chown deploy:almanach /opt/almanach/app
chmod 755 /opt/almanach/app

mkdir -p /opt/almanach/bin
chown almanach:almanach /opt/almanach/bin
chmod 755 /opt/almanach/bin

mkdir -p /var/lib/almanach/data
chown almanach:almanach /var/lib/almanach/data
chmod 750 /var/lib/almanach/data

mkdir -p /opt/almanach/scripts
chown deploy:almanach /opt/almanach/scripts
chmod 755 /opt/almanach/scripts

mkdir -p /home/deploy/.ssh
chown deploy:deploy /home/deploy/.ssh
chmod 700 /home/deploy/.ssh

# Install dependencies
apt-get update
apt-get install -y build-essential pkg-config libssl-dev curl git

# Install Rust via rustup for the deploy user if not already installed
if ! su - deploy -c 'command -v cargo' >/dev/null 2>&1; then
  su - deploy -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
  echo "Installed Rust for deploy user"
fi

# Determine if this script is being run from inside an existing Almanach01 repo
IN_REPO=false
if [[ -d "${REPO_ROOT}/.git" ]]; then
  REMOTE_URL=$(git -C "${REPO_ROOT}" remote get-url origin 2>/dev/null || true)
  if [[ "${REMOTE_URL}" == *"cano-lab/Almanach01"* ]]; then
    IN_REPO=true
  fi
fi

# Populate app directory if it's empty
if [[ -z "$(ls -A /opt/almanach/app 2>/dev/null || true)" ]]; then
  if [[ "${IN_REPO}" == true ]]; then
    echo "Copying existing repo from ${REPO_ROOT} to /opt/almanach/app..."
    cp -a "${REPO_ROOT}/." /opt/almanach/app
    chown -R deploy:almanach /opt/almanach/app
  else
    git clone https://github.com/cano-lab/Almanach01.git /opt/almanach/app
    chown -R deploy:almanach /opt/almanach/app
    echo "Cloned repo into /opt/almanach/app"
  fi
fi

# Symlink data directory
if [[ ! -e /opt/almanach/app/data ]]; then
  ln -s /var/lib/almanach/data /opt/almanach/app/data
  echo "Created symlink /opt/almanach/app/data -> /var/lib/almanach/data"
elif [[ -L /opt/almanach/app/data ]]; then
  echo "Symlink /opt/almanach/app/data already exists"
else
  echo "WARNING: /opt/almanach/app/data exists and is not a symlink. Skipping."
fi

# Copy systemd service file
SERVICE_SRC="${REPO_ROOT}/scripts/almanach.service"
if [[ -f "${SERVICE_SRC}" ]]; then
  cp "${SERVICE_SRC}" /etc/systemd/system/almanach-orchestrator.service
  echo "Copied systemd service file"
else
  echo "ERROR: Service file not found at ${SERVICE_SRC}"
  exit 1
fi

# Copy deploy script so it can be invoked by GitHub Actions and manually
DEPLOY_SRC="${REPO_ROOT}/scripts/deploy.sh"
if [[ -f "${DEPLOY_SRC}" ]]; then
  cp "${DEPLOY_SRC}" /opt/almanach/scripts/deploy.sh
  chmod +x /opt/almanach/scripts/deploy.sh
  chown deploy:almanach /opt/almanach/scripts/deploy.sh
  echo "Copied deploy script to /opt/almanach/scripts/deploy.sh"
else
  echo "ERROR: Deploy script not found at ${DEPLOY_SRC}"
  exit 1
fi

systemctl daemon-reload
systemctl enable almanach-orchestrator

# Configure sudoers for deploy user
cat <<EOF > /etc/sudoers.d/almanach-deploy
deploy ALL=(root) NOPASSWD: /bin/systemctl start almanach-orchestrator
deploy ALL=(root) NOPASSWD: /bin/systemctl stop almanach-orchestrator
deploy ALL=(root) NOPASSWD: /bin/systemctl restart almanach-orchestrator
deploy ALL=(root) NOPASSWD: /bin/systemctl status almanach-orchestrator
EOF
chmod 440 /etc/sudoers.d/almanach-deploy
visudo -c

echo ""
echo "========================================"
echo "VPS provisioning complete!"
echo "========================================"
echo ""
echo "Next steps:"
echo "  1. Copy your .env file to /opt/almanach/.env"
echo "  2. Copy/seed any data to /var/lib/almanach/data/"
echo "  3. Add the GitHub Actions SSH public key to /home/deploy/.ssh/authorized_keys"
echo "  4. Run: systemctl start almanach-orchestrator"
echo ""
