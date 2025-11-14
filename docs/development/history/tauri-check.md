---
description: "Verify Tauri development environment (Rust, Node, Tauri CLI) and test desktop build pipeline."
allowed-tools:
  - Bash
model: haiku
---

# Tauri Environment Check

Verifying Tauri desktop development environment...

## Step 1: Check Rust Installation

```bash
# Check Rust version (should be 1.70+)
rustc --version

# Check Cargo version
cargo --version
```

## Step 2: Check Node.js Installation

```bash
# Check Node version (should be 18+)
node --version

# Check npm version
npm --version
```

## Step 3: Check Tauri CLI

```bash
# Check if Tauri CLI is installed
cargo tauri --version || echo "❌ Tauri CLI not installed. Run: cargo install tauri-cli"
```

## Step 4: Verify Project Structure

```bash
# Check if src-tauri directory exists
if [ -d "/workspace/src-tauri" ]; then
  echo "✅ src-tauri directory found"
else
  echo "❌ src-tauri directory missing"
fi

# Check if Tauri config exists
if [ -f "/workspace/src-tauri/tauri.conf.json" ]; then
  echo "✅ Tauri config found"
else
  echo "❌ Tauri config missing"
fi
```

## Step 5: Check Dependencies

```bash
cd /workspace

# Check if portal dependencies are installed
if [ -d "apps/portal/node_modules" ]; then
  echo "✅ Portal dependencies installed"
else
  echo "⚠️  Portal dependencies not installed. Run: cd apps/portal && npm install"
fi

# Check if simulation compiles
cd apps/simulation
cargo check 2>&1 | tail -n 10
```

## Step 6: Test Desktop Build (Debug)

```bash
# Try a debug build (faster than release)
cd /workspace
cargo tauri build --debug 2>&1 | tail -n 20

# Check if build succeeded
if [ $? -eq 0 ]; then
  echo "✅ Tauri desktop build succeeded!"
else
  echo "❌ Tauri desktop build failed. Check errors above."
fi
```

## Summary

Run all checks above and report:
- ✅ What's working
- ❌ What's broken
- ⚠️  What needs attention

Suggest fixes for any failures.
