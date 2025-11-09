#!/bin/bash
# PreToolUse hook: Enforce TDD by running tests before Edit/Write operations
#
# This hook blocks Edit and Write operations if tests fail, ensuring
# Claude cannot break working code without tests catching it immediately.

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
if [[ "$FILE_PATH" =~ \.(md|json|toml|txt|html)$ ]]; then
  exit 0  # Allow documentation and config edits without test check
fi

echo "⚠️  TDD Enforcement: Running tests before allowing code edit..." >&2
echo "   File: $FILE_PATH" >&2
echo "" >&2

# Change to workspace root (where package.json is)
cd /workspace/apps/portal || exit 2

# Run tests and capture output
TEST_OUTPUT=$(npm test 2>&1)
TEST_EXIT_CODE=$?

if [ $TEST_EXIT_CODE -eq 0 ]; then
  echo "✅ Tests passed! Edit allowed." >&2
  echo "" >&2
  exit 0  # Allow the edit
else
  echo "❌ TESTS FAILED! Edit blocked." >&2
  echo "" >&2
  echo "Test output:" >&2
  echo "$TEST_OUTPUT" >&2
  echo "" >&2
  echo "🚫 Fix failing tests before making code changes!" >&2
  echo "   Run: npm test" >&2
  exit 2  # Block the edit
fi
