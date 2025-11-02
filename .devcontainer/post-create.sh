#!/bin/bash
set -e

echo "Running post-create setup..."

# Install Claude Code CLI globally
npm install -g @anthropic-ai/claude-code

# Fix ownership of cargo registry
sudo chown -R vscode:vscode /usr/local/cargo

# Fix ownership of simulation target directory
sudo chown -R vscode:vscode /workspace/apps/simulation/target

echo "Post-create setup complete!"