#!/bin/bash
# PreCommit hook: Code Consistency and Quality Checks
#
# This hook runs before commits to ensure:
# 1. Commit messages follow convention
# 2. No console.log() in production code
# 3. No hardcoded secrets
#
# MODE: Advisory + Blocking (blocks on critical issues)

set -euo pipefail

# Get staged files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)

# If no staged files, exit
if [[ -z "$STAGED_FILES" ]]; then
  exit 0
fi

WARNINGS=()
CRITICAL_ERRORS=()

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "🔍  PRE-COMMIT CONSISTENCY CHECK" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "" >&2

# Check 1: console.log() detection (warning)
echo "Checking for console.log()..." >&2
for file in $STAGED_FILES; do
  if [[ "$file" =~ \.(ts|tsx|js|jsx)$ ]] && [[ ! "$file" =~ \.test\. ]] && [[ ! "$file" =~ \.spec\. ]]; then
    if git diff --cached "$file" | grep -qE "^\+.*console\.log\("; then
      WARNINGS+=("console.log() found in: $file")
    fi
  fi
done

# Check 2: Hardcoded secrets (critical)
echo "Checking for hardcoded secrets..." >&2
SECRET_PATTERNS=(
  "API_KEY.*=.*['\"][A-Za-z0-9_-]{20,}['\"]"
  "SECRET.*=.*['\"][A-Za-z0-9_-]{20,}['\"]"
  "PASSWORD.*=.*['\"][^'\"]+['\"]"
  "TOKEN.*=.*['\"][A-Za-z0-9_-]{20,}['\"]"
  "aws_access_key"
  "aws_secret_key"
  "PRIVATE_KEY"
)

for file in $STAGED_FILES; do
  if [[ "$file" =~ \.(ts|tsx|js|jsx|rs|env|yml|yaml)$ ]]; then
    for pattern in "${SECRET_PATTERNS[@]}"; do
      if git diff --cached "$file" | grep -qiE "^\+.*$pattern"; then
        CRITICAL_ERRORS+=("Potential secret detected in: $file (pattern: $pattern)")
      fi
    done
  fi
done

# Check 3: Commit message format (advisory)
COMMIT_MSG_FILE=".git/COMMIT_EDITMSG"
if [[ -f "$COMMIT_MSG_FILE" ]]; then
  COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

  # Check if commit follows conventional format (feat:, fix:, docs:, etc.)
  if ! echo "$COMMIT_MSG" | grep -qE "^(feat|fix|docs|style|refactor|test|chore|perf)(\(.+\))?:"; then
    WARNINGS+=("Commit message doesn't follow convention (feat:, fix:, docs:, etc.)")
  fi
fi

# Report warnings
if [ ${#WARNINGS[@]} -gt 0 ]; then
  echo "⚠️  WARNINGS (non-blocking):" >&2
  for warning in "${WARNINGS[@]}"; do
    echo "  - $warning" >&2
  done
  echo "" >&2
fi

# Report critical errors
if [ ${#CRITICAL_ERRORS[@]} -gt 0 ]; then
  echo "❌ CRITICAL ERRORS (blocking commit):" >&2
  for error in "${CRITICAL_ERRORS[@]}"; do
    echo "  - $error" >&2
  done
  echo "" >&2
  echo "🚫 Commit blocked! Remove secrets before committing." >&2
  echo "" >&2
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
  exit 1  # Block commit
fi

# All checks passed
if [ ${#WARNINGS[@]} -eq 0 ]; then
  echo "✅ All checks passed!" >&2
else
  echo "✅ Critical checks passed (warnings noted above)" >&2
fi

echo "" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "" >&2

# Allow commit (warnings don't block)
exit 0
