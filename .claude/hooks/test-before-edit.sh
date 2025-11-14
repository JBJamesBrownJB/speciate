#!/bin/bash
# PreToolUse hook: Enforce TDD by running tests before Edit/Write operations
#
# This hook validates code quality by running appropriate tests based on file type.
# Can be bypassed with SKIP_TEST_HOOK=1 environment variable for architectural refactoring.

# Read the tool use JSON from stdin
TOOL_USE=$(cat)

# Extract tool name from JSON (Edit or Write)
TOOL_NAME=$(echo "$TOOL_USE" | jq -r '.tool // empty')

# Only check tests for Edit and Write operations
if [[ "$TOOL_NAME" != "Edit" && "$TOOL_NAME" != "Write" ]]; then
  exit 0  # Allow other tools without test check
fi

# Extract file path to see what's being modified
FILE_PATH=$(echo "$TOOL_USE" | jq -r '.parameters.file_path // empty')

# Skip test check for non-code files (markdown, config, etc.)
if [[ "$FILE_PATH" =~ \.(md|json|toml|txt|html|yml|yaml|sh)$ ]]; then
  exit 0  # Allow documentation and config edits without test check
fi

# Check for skip flag (useful during architectural refactoring)
if [[ "$SKIP_TEST_HOOK" == "1" ]]; then
  echo "⚠️  Test hook skipped (SKIP_TEST_HOOK=1)" >&2
  exit 0
fi

# Detect file type and determine appropriate test command
if [[ "$FILE_PATH" =~ \.rs$ ]]; then
  # Rust file - run cargo tests
  TEST_TYPE="Rust"
  TEST_CMD="cargo test"
  WORK_DIR="/workspace/apps/simulation"

  # Check if we're in simulation directory
  if [[ ! "$FILE_PATH" =~ ^/workspace/apps/simulation ]]; then
    # Rust file outside simulation? Skip for now
    exit 0
  fi

elif [[ "$FILE_PATH" =~ \.(ts|tsx|js|jsx)$ ]]; then
  # TypeScript/JavaScript file - run npm tests
  TEST_TYPE="TypeScript"
  TEST_CMD="npm test"
  WORK_DIR="/workspace/apps/portal"

  # Check if we're in portal directory
  if [[ ! "$FILE_PATH" =~ ^/workspace/apps/portal ]]; then
    # TS file outside portal? Skip for now
    exit 0
  fi

else
  # Unknown file type, skip test check
  exit 0
fi

echo "⚠️  TDD Enforcement: Running $TEST_TYPE tests before allowing code edit..." >&2
echo "   File: $FILE_PATH" >&2
echo "   Working dir: $WORK_DIR" >&2
echo "" >&2

# Change to appropriate directory
if [[ ! -d "$WORK_DIR" ]]; then
  echo "⚠️  Warning: $WORK_DIR not found, skipping tests" >&2
  exit 0
fi

cd "$WORK_DIR" || exit 0

# Run tests with timeout (max 2 minutes)
TEST_OUTPUT=$(timeout 120 $TEST_CMD 2>&1)
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -eq 124 ]; then
  echo "⚠️  WARNING: Tests timed out (> 2 minutes). Edit allowed but tests may be broken." >&2
  echo "" >&2
  exit 0  # Allow edit but warn

elif [ $TEST_EXIT_CODE -eq 0 ]; then
  echo "✅ $TEST_TYPE tests passed! Edit allowed." >&2
  echo "" >&2
  exit 0  # Allow the edit

else
  echo "❌ $TEST_TYPE TESTS FAILED! Edit blocked." >&2
  echo "" >&2
  echo "Test output (last 30 lines):" >&2
  echo "$TEST_OUTPUT" | tail -n 30 >&2
  echo "" >&2
  echo "🚫 Fix failing tests before making code changes!" >&2
  echo "   Run: cd $WORK_DIR && $TEST_CMD" >&2
  echo "   Or skip: SKIP_TEST_HOOK=1 (not recommended)" >&2
  exit 2  # Block the edit
fi
