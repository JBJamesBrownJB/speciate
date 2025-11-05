#!/bin/bash
set -e

echo "Running post-create setup..."

# Install Claude Code CLI globally
npm install -g @anthropic-ai/claude-code

# Install NATS CLI for debugging
echo "Installing NATS CLI..."
curl -sf https://binaries.nats.dev/nats-io/natscli/nats@latest | sh
sudo mv nats /usr/local/bin/

# Install wscat for websocket debugging
echo "Installing wscat for WebSocket debugging"
npm install -g wscat

# Fix ownership of cargo registry
sudo chown -R vscode:vscode /usr/local/cargo

# Fix ownership of simulation target directory
sudo chown -R vscode:vscode /workspace/apps/simulation/target

# Fix Docker socket group permissions to match host
echo "Synchronizing Docker group GID with host..."
DOCKER_SOCK_GID=$(stat -c '%g' /var/run/docker.sock 2>/dev/null || echo "")
if [ -n "$DOCKER_SOCK_GID" ]; then
    echo "Host Docker socket GID: $DOCKER_SOCK_GID"
    echo "Updating container docker group to match..."
    sudo groupmod -g "$DOCKER_SOCK_GID" docker
    sudo usermod -aG docker vscode
    echo "Docker group synchronized. Group membership will be active after shell restart."
else
    echo "Warning: Could not detect Docker socket GID. Docker may not be available."
fi

echo "Post-create setup complete!"