#!/bin/bash
# PostToolUse hook: Documentation Review Reminder
#
# Triggers after code changes to remind Claude to review relevant documentation
# for accuracy, completeness, clarity, and consistency.
#
# MODE: Smart detection + Automated checks + Advisory (non-blocking)

set -euo pipefail

# Read tool use JSON from stdin
TOOL_USE=$(cat)

# Extract tool info
TOOL_NAME=$(echo "$TOOL_USE" | jq -r '.tool // empty')
FILE_PATH=$(echo "$TOOL_USE" | jq -r '.parameters.file_path // empty')

# Only check after Edit/Write operations
if [[ "$TOOL_NAME" != "Edit" && "$TOOL_NAME" != "Write" ]]; then
  exit 0
fi

# Skip if no file path
if [[ -z "$FILE_PATH" ]]; then
  exit 0
fi

# Skip test files
if [[ "$FILE_PATH" =~ \.test\. ]] || [[ "$FILE_PATH" =~ \.spec\. ]]; then
  exit 0
fi

# Only check code files
if [[ ! "$FILE_PATH" =~ \.(ts|tsx|js|jsx|rs|md)$ ]]; then
  exit 0
fi

# For markdown files, check for simple/readable formatting
if [[ "$FILE_PATH" =~ \.md$ ]]; then
  # Check for overly complex markdown that's hard to read outside preview mode
  ISSUES=()

  # Check for excessive indentation (more than 2 levels)
  if grep -qE '^        ' "$FILE_PATH" 2>/dev/null; then
    ISSUES+=("Deep indentation detected (4+ levels) - hard to read in plain text")
  fi

  # Check for emoji in headings (distracting in plain text)
  if grep -qE '^#+.*[🎯📚🧭✨🔬🌱💡🚀]' "$FILE_PATH" 2>/dev/null; then
    ISSUES+=("Emoji in headings - remove for cleaner plain text readability")
  fi

  # Check for unformatted code blocks (should use ``` fences)
  if grep -qE '^    [A-Za-z].*=.*[A-Za-z]' "$FILE_PATH" 2>/dev/null; then
    ISSUES+=("Indented code blocks detected - use ``` fenced code blocks instead")
  fi

  # If issues found, show advisory
  if [ ${#ISSUES[@]} -gt 0 ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
    echo "📄  MARKDOWN READABILITY CHECK" >&2
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
    echo "" >&2
    echo "File: $FILE_PATH" >&2
    echo "" >&2
    echo "Issues detected (use simple markdown for plain-text readability):" >&2
    for issue in "${ISSUES[@]}"; do
      echo "  ⚠ $issue" >&2
    done
    echo "" >&2
    echo "Guidelines for simple, readable markdown:" >&2
    echo "  • Use headings (#, ##, ###) instead of emoji" >&2
    echo "  • Use bullet lists (-) with max 2 levels of nesting" >&2
    echo "  • Use ``` code fences instead of 4-space indentation" >&2
    echo "  • Use bold (**text**) for emphasis, not complex formatting" >&2
    echo "  • Keep it readable both in preview and plain text" >&2
    echo "" >&2
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
  fi

  exit 0
fi

# --- SMART DETECTION: Map changed file to relevant documentation ---
RELEVANT_DOCS=()

# Portal domain layer (Camera, Viewport, Interpolator, Creature)
if [[ "$FILE_PATH" =~ apps/portal/src/domain ]]; then
  RELEVANT_DOCS+=("apps/portal/ARCHITECTURE.md")
fi

# Portal rendering layer
if [[ "$FILE_PATH" =~ apps/portal/src/rendering ]]; then
  RELEVANT_DOCS+=("apps/portal/ARCHITECTURE.md")
fi

# Portal infrastructure (WebSocket, SpritePool)
if [[ "$FILE_PATH" =~ apps/portal/src/(core|infrastructure) ]]; then
  RELEVANT_DOCS+=("apps/portal/ARCHITECTURE.md")
  RELEVANT_DOCS+=("apps/portal/README.md")
fi

# Portal constants - affects project instructions
if [[ "$FILE_PATH" =~ apps/portal/src/core/constants ]]; then
  RELEVANT_DOCS+=("CLAUDE.md")
  RELEVANT_DOCS+=("apps/portal/ARCHITECTURE.md")
fi

# Simulation creature/genetics code - DNA-driven design
if [[ "$FILE_PATH" =~ (components\.rs|genetics|spawner|creature) ]]; then
  RELEVANT_DOCS+=("docs/biology/dna-driven-design.md")
  RELEVANT_DOCS+=("docs/biology/biology-notes.md")
fi

# Simulation ECS systems
if [[ "$FILE_PATH" =~ apps/simulation/src/systems ]]; then
  RELEVANT_DOCS+=("apps/simulation/README.md")
fi

# No relevant docs? Exit silently
if [ ${#RELEVANT_DOCS[@]} -eq 0 ]; then
  exit 0
fi

# Remove duplicates
UNIQUE_DOCS=($(echo "${RELEVANT_DOCS[@]}" | tr ' ' '\n' | sort -u | tr '\n' ' '))

# --- OUTPUT: Show advisory reminder ---
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "📚  DOCUMENTATION REVIEW REMINDER" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "" >&2
echo "Code changes in: $FILE_PATH" >&2
echo "" >&2
echo "Potentially affected documentation:" >&2
for doc in "${UNIQUE_DOCS[@]}"; do
  echo "  - $doc" >&2
done
echo "" >&2
echo "Please verify:" >&2
echo "  ✓ Accuracy - Does it match current code?" >&2
echo "  ✓ Completeness - Are new features documented?" >&2
echo "  ✓ Clarity - Is it easy to understand?" >&2
echo "  ✓ Consistency - Is terminology consistent?" >&2
echo "" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2

# Exit 0: Non-blocking (advisory only)
exit 0
