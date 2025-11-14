---
description: "Consult zoologist-tom agent for DNA/trait design guidance and auto-log result in biology-notes.md."
allowed-tools:
  - Task
  - Read
  - Edit
model: sonnet
---

# DNA Consultation Workflow

You are helping the user consult with the zoologist-tom agent for biological guidance on creature traits and DNA design.

## Step 1: Parse User Request

The user's consultation request is:
"$ARGUMENTS"

## Step 2: Dispatch to Zoologist

Use the Task tool to dispatch to the **zoologist-tom** agent with this request:

```
subagent_type: zoologist-tom
prompt: $ARGUMENTS

Ask the zoologist to provide:
1. Realistic range/bounds for the trait
2. How it scales with other attributes
3. Trade-offs and ecological implications
4. Suggested gene implementation
```

## Step 3: Log Consultation

After receiving the zoologist's response, append it to `/workspace/docs/biology/biology-notes.md` using this format:

```markdown
### [Current Date] - [Trait Name]

**Question:** $ARGUMENTS

**Zoologist Response:**
[Full response from zoologist-tom]

**Implementation Status:** Pending / In Progress / Complete

---
```

## Step 4: Summary

Provide the user with:
1. The zoologist's key recommendations
2. Confirmation that the consultation was logged
3. Next steps for implementation

---

**Note:** This command ensures all biological decisions are:
- Informed by expert consultation
- Permanently documented
- Traceable for future reference
