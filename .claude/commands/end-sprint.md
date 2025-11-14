---
description: Initiates the Sprint Closure workflow. Automatically detects the current sprint from the Git branch, generates the summary, logs notes, tidies up sprint documents, and instructs the human on final merge steps.
allowed-tools:
  - Bash
  - Read
  - Write
  - Task
model: sonnet
---

# Sprint Closure Workflow: QA, Documentation, and Tidy Up

You have been called to finalize the sprint. This workflow includes QA verification, documentation generation, and cleanup.

## 1. 🔍 Identify Current Sprint

1.  Execute `!git rev-parse --abbrev-ref HEAD` to get the current branch name.
2.  Parse the output (e.g., "feat/sprint-1-blueprints") to extract **only** the sprint name (e.g., "sprint-1-blueprints").
3.  Store this as the **`CURRENT_SPRINT_NAME`** for all following steps.

## 2. ✅ QA Verification (Pre-Merge Check)

Before proceeding with sprint closure, verify code quality:

1.  **Run Test Suite:** Execute `/test-all` command to verify all tests pass
2.  **Call QA Agent:** Use the Task tool to invoke **qa-karen** agent for code review:
    - Review recent commits on current branch
    - Validate architectural compliance
    - Check for security issues
    - Verify code style and idioms
3.  **If QA Fails:** STOP and report issues to user. Sprint cannot close until QA passes.
4.  **If QA Passes:** Proceed to documentation step.

**Note:** QA verification is MANDATORY. Never skip this step.

## 3. 📝 Final Review and Documentation

1.  **Summary Creation:** Read all documentation in the `/SPRINT_DOCS` folder and generate a concise **SPRINT_SUMMARY.md** file. This summary **MUST** include:
    * The original **Sprint Goal** and **Key Outcomes**.
    * A final list of **Completed Tasks** and **Remaining Work**.
    * A section for **Retrospective/Lessons Learned**.
2.  **Move Summary:** **Convert the `CURRENT_SPRINT_NAME` to lowercase.** Move the generated **SPRINT_SUMMARY.md** into the required format: **`[current_sprint_name]_summary.md`** and place it in the **`/sprint_summaries`** folder in the root directory.

## 4. 🧹 Final Cleanup

1.  **Log:** Log the successful closure in **SPRINT_DOCS/SESSION_LOG.md**.
2.  **Remove Sprint Docs:** Execute `!rm -rf SPRINT_DOCS` to ensure no planning documents are left in the main workspace.

## 5. 🌲 Next Action: Human Hand-off

Your work is complete. The human developer must now perform the final Git actions.

**Final Instruction to Human:**

> "The documentation cleanup is complete. The **`$CURRENT_SPRINT_NAME`** summary is saved in **`/sprint_summaries`**.
>
> You must now manually execute the Git commands to merge and tidy up:
> 1.  `git checkout main`
> 2.  `git merge --squash feat/$CURRENT_SPRINT_NAME`
> 3.  `SECRET_AC_CODE_HUMAN_ONLY=true git commit -m "feat(sprint-end): Final code from $CURRENT_SPRINT_NAME"`
> 4.  `git branch -d feat/$CURRENT_SPRINT_NAME`
> 5.  **Clean up Remote Branch:** `git push origin --delete feat/$CURRENT_SPRINT_NAME`"