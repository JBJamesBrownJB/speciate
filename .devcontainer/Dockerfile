# Base image with Rust pre-installed
FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# NOTE: Node.js v24 is provided via devcontainer features
# NOTE: PostgreSQL client tools will be added when database integration is needed
# postgresql-client, libpq-dev

# Install essential build tools
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    vim \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Rust tooling
RUN rustup component add rustfmt clippy rust-analyzer

# Install cargo-watch for hot reload during development
RUN cargo install cargo-watch

# Create workspace directory
RUN mkdir -p /workspace
WORKDIR /workspace

# Configure git for the container
RUN git config --global --add safe.directory /workspace

# Switch to vscode user (created by base image)
USER vscode

# Verify installations
RUN rustc --version \
    && cargo --version
