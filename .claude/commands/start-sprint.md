---
description: Initiates a new sprint workflow, elicits goals/constraints, performs Git pre-checks, creates the feature branch, and drafts the initial plan.
allowed-tools:
  - Bash
  - Read
  - Write
model: sonnet
---

# Sprint Initialization Wizard

You have been called to initiate a new sprint. The human user must provide the following:

1.  **Sprint Goal:** (The single, overarching objective.)
2.  **Key Outcomes:** (1-3 measurable deliverables.)
3.  **Key Constraints:** (Time, budget, scope limits.)

Once received, execute the following steps in order.

## 1. 📝 Design and Validation

Based on the provided Goal, Outcome, and Constraints:
* Propose a **snappy, punchy sprint name** (e.g., `sprint-2-data-layer`). Use this for the branch name.
* Check that the proposed sprint name does not conflict with existing branches (`!git branch --list feat/$SPRINT_NAME*`).

## 2. 🛡️ Pre-Flight Check (Mandatory)

Execute the following commands and report the status. If any check fails, **STOP IMMEDIATELY and report the error**.

1.  **Check for Uncommitted Changes:** `!git status --porcelain`
2.  **Verify Main Branch:** `!git rev-parse --abbrev-ref HEAD` (Must confirm it is 'main').
3.  **Check for Empty Docs Directory:** Verify the **SPRINT_DOCS** folder is empty or non-existent (`!ls -A SPRINT_DOCS`).
4.  **Verify Development Environment:**
    - Check Rust: `rustc --version`
    - Check Node: `node --version`
    - Check npm: `npm --version`
    - Warn if any missing, but don't block sprint start

## 3. 🌲 Setup and Branching

Once all checks pass:

1.  **Create Branch:** Execute `!git checkout -b feat/$SPRINT_NAME`.
2.  **Placeholder Plan:** Create a new file, **SPRINT_DOCS/SPRINT_PLAN_$SPRINT_NAME.md**, and populate it with a header, the elicited Goal, Outcomes, and Constraints.
3.  **Backlog Initialization:** Update **SPRINT_DOCS/SPRINT_BACKLOG.md** by adding a new section header for the sprint.
4.  **Log:** Log the successful branch creation and folder initialization in **SPRINT_DOCS/SESSION_LOG.md**.
5.  **Documentation Tracking:** If the sprint is based on a design doc from `docs/*/`, note in the sprint plan that the doc should be moved to `docs/*/done/` when the sprint completes.