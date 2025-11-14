---
description: "Display current sprint overview (branch, goals, commits, open tasks)."
allowed-tools:
  - Bash
  - Read
model: haiku
---

# Sprint Status Overview

Generating current sprint status...

## Step 1: Detect Current Sprint

```bash
# Get current branch name
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)

echo "========================================="
echo "CURRENT SPRINT"
echo "========================================="
echo "Branch: $CURRENT_BRANCH"
echo ""

# Extract sprint number from branch (e.g., feat/sprint-8-player-interaction → Sprint 8)
if [[ "$CURRENT_BRANCH" =~ sprint-([0-9]+) ]]; then
  SPRINT_NUM="${BASH_REMATCH[1]}"
  echo "Sprint: #$SPRINT_NUM"
else
  echo "Sprint: Unknown (not on a sprint branch)"
fi

echo ""
```

## Step 2: Sprint Goals

```bash
# Try to read SPRINT_PLAN.md if it exists
if [ -f "/workspace/SPRINT_PLAN.md" ]; then
  echo "========================================="
  echo "SPRINT GOALS"
  echo "========================================="
  cat /workspace/SPRINT_PLAN.md | head -n 20
  echo ""
else
  echo "⚠️  No SPRINT_PLAN.md found. Use /start-sprint to create one."
  echo ""
fi
```

## Step 3: Recent Commits

```bash
echo "========================================="
echo "RECENT COMMITS (Last 5)"
echo "========================================="

git log --oneline --max-count=5

echo ""
```

## Step 4: Uncommitted Changes

```bash
echo "========================================="
echo "UNCOMMITTED CHANGES"
echo "========================================="

git status --short

if [ -z "$(git status --short)" ]; then
  echo "✅ Working directory clean (no uncommitted changes)"
else
  echo ""
  echo "⚠️  Uncommitted changes detected. Consider committing before switching tasks."
fi

echo ""
```

## Step 5: Test Status

```bash
echo "========================================="
echo "QUICK TEST CHECK"
echo "========================================="

# Run portal tests (quick check, don't block on failure)
cd /workspace/apps/portal
npm test -- --run --reporter=basic 2>&1 | tail -n 5
PORTAL_STATUS=$?

if [ $PORTAL_STATUS -eq 0 ]; then
  echo "✅ Portal tests passing"
else
  echo "❌ Portal tests failing (run /test-all for details)"
fi

echo ""
```

## Step 6: Summary

```bash
echo "========================================="
echo "SUMMARY"
echo "========================================="
echo ""
echo "Sprint: $CURRENT_BRANCH"
echo "Last commit: $(git log -1 --format='%s')"
echo "Author: $(git log -1 --format='%an')"
echo "Date: $(git log -1 --format='%ar')"
echo ""
echo "Next steps:"
echo "1. Review SPRINT_PLAN.md for remaining tasks"
echo "2. Run /test-all to verify full test suite"
echo "3. Use 'npm run dev' to verify desktop build"
echo ""
```

---

**Note:** This command provides a quick overview of sprint progress. For detailed planning, use /start-sprint or consult pm-pam.
