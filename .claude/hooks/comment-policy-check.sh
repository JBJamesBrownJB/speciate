#!/bin/bash
# PreToolUse hook: Enforce "Death to Comments" policy
#
# Blocks commits with:
# - Multi-line comments (/// or /**/ or //!)
# - JSDoc (@param, @returns)
# - Rustdoc doc comments
#
# Allows:
# - Single-line constant comments (e.g., "pub const FOO: f32 = 1.0; // Brief description")
# - TODO markers
# - Shell script headers

TOOL_USE=$(cat)
TOOL_NAME=$(echo "$TOOL_USE" | jq -r '.tool // empty')

# Only check Edit and Write operations
if [[ "$TOOL_NAME" != "Edit" && "$TOOL_NAME" != "Write" ]]; then
  exit 0
fi

# Extract file path and content based on tool type
FILE_PATH=$(echo "$TOOL_USE" | jq -r '.parameters.file_path // empty')

if [[ "$TOOL_NAME" == "Edit" ]]; then
  CONTENT=$(echo "$TOOL_USE" | jq -r '.parameters.new_string // empty')
elif [[ "$TOOL_NAME" == "Write" ]]; then
  CONTENT=$(echo "$TOOL_USE" | jq -r '.parameters.content // empty')
else
  exit 0
fi

# Skip non-code files
if [[ "$FILE_PATH" =~ \.(md|json|toml|txt|html|yml|yaml)$ ]]; then
  exit 0
fi

# Allow shell script headers (concise only)
if [[ "$FILE_PATH" =~ \.sh$ ]]; then
  # Allow shell scripts but warn if header is >3 lines
  HEADER_LINES=$(echo "$CONTENT" | grep '^#' | head -n 10 | wc -l)
  if [[ $HEADER_LINES -gt 3 ]]; then
    echo "⚠️  Warning: Shell script header is verbose ($HEADER_LINES comment lines)" >&2
    echo "   Keep it to 1-3 lines maximum (functional description only)" >&2
    echo "   File: $FILE_PATH" >&2
    echo "" >&2
  fi
  exit 0  # Allow shell scripts
fi

# Detect violations
VIOLATIONS=()

# Check for Rustdoc module comments (//!)
if echo "$CONTENT" | grep -q '^ *//!'; then
  VIOLATIONS+=("Rustdoc module comments (//!) are banned")
fi

# Check for Rustdoc item comments (///)
if echo "$CONTENT" | grep -q '^ *///'; then
  VIOLATIONS+=("Rustdoc item comments (///) are banned")
fi

# Check for JSDoc comments (/** */)
if echo "$CONTENT" | grep -q '/\*\*'; then
  VIOLATIONS+=("JSDoc comments (/** */) are banned")
fi

# Check for multi-line block comments (/* */) - but allow single-line
if echo "$CONTENT" | grep -Pzo '/\*(?!.*\*/.*$)' > /dev/null 2>&1; then
  VIOLATIONS+=("Multi-line block comments (/* ... */) are banned")
fi

# Check for @param, @returns, @example in comments
if echo "$CONTENT" | grep -q '@param\|@returns\|@example\|@description'; then
  VIOLATIONS+=("JSDoc tags (@param, @returns, etc.) are banned")
fi

# Check for comment blocks (3+ consecutive // lines that aren't TODOs)
CONSECUTIVE_COMMENTS=$(echo "$CONTENT" | grep -v 'TODO\|FIXME' | awk '
  /^ *\/\// { count++; if (count > 2) exit 1 }
  !/^ *\/\// { count=0 }
')
if [[ $? -eq 1 ]]; then
  VIOLATIONS+=("Comment blocks (3+ consecutive lines) detected - move to docs/")
fi

# If violations found, block the edit
if [[ ${#VIOLATIONS[@]} -gt 0 ]]; then
  echo "❌ COMMENT POLICY VIOLATION - Edit blocked!" >&2
  echo "" >&2
  echo "File: $FILE_PATH" >&2
  echo "" >&2
  echo "Violations found:" >&2
  for violation in "${VIOLATIONS[@]}"; do
    echo "  ❌ $violation" >&2
  done
  echo "" >&2
  echo "🚫 DEATH TO COMMENTS!" >&2
  echo "" >&2
  echo "Allowed:" >&2
  echo "  ✅ Single-line constant comments: pub const FOO: f32 = 1.0; // Brief concept" >&2
  echo "  ✅ TODO markers: // TODO(DNA): Migrate to gene expression" >&2
  echo "  ✅ Shell headers: # Brief functional description (1-3 lines max)" >&2
  echo "" >&2
  echo "Banned:" >&2
  echo "  ❌ Doc comments (///, //!, /***/)" >&2
  echo "  ❌ Multi-line comments" >&2
  echo "  ❌ JSDoc tags (@param, @returns)" >&2
  echo "  ❌ Comment blocks (3+ lines)" >&2
  echo "" >&2
  echo "Fix:" >&2
  echo "  1. Refactor code to be self-documenting" >&2
  echo "  2. Move rationale to /docs/" >&2
  echo "  3. Shorten to one line (constants only)" >&2
  echo "" >&2
  echo "See: CLAUDE.md - Code Documentation Standards" >&2
  exit 2  # Block the edit
fi

# No violations - allow edit
exit 0
