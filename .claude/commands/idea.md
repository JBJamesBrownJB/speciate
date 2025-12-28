---
description: "Receives, records and organises new ideas into docs/"
allowed-tools:
  - Read
  - Grep
  - Glob
  - Write
  - Edit
  - Task
  - AskUserQuestion
model: sonnet
---

# Idea Capture Workflow

You are helping the user capture and organize ideas into the project documentation.

## The Input

"$ARGUMENTS"

---

## Step 0: Detect Multiple Ideas

If the input contains multiple distinct ideas (look for bullet points, numbered lists, or clearly separate concepts):

1. List each idea you detected
2. Confirm with the user: "I found N ideas - shall I process them separately?"
3. Process each idea through Steps 1-5 independently
4. Create separate files for each

**One idea = one file. Never combine unrelated concepts.**

---

## Step 1: Clarify the Idea

Ask 2-4 focused clarifying questions:
- **What problem does this solve?** (or what opportunity does it enable?)
- **Category** - biology, gameplay, performance, visuals, architecture?
- **Scope** - Small tweak or major feature?
- **Dependencies** - What must exist first?

Use AskUserQuestion with concise options where possible.

---

## Step 2: Consult Domain Experts

Based on category, consult the appropriate sub-agent using the Task tool:

| Category | Agent | What to Ask |
|----------|-------|-------------|
| Biology/behavior/DNA/ecology | `zoologist-tom` | Biological plausibility, real-world analogues, trait trade-offs |
| ECS/components/parallelization | `ecs-emma` | Data layout, archetype implications, DOD patterns |
| Performance/profiling/Hz | `instrumentation-ian` | Measurement approach, bottleneck analysis |
| Visuals/shaders/rendering | `shader-sarah` | GPU considerations, visual feasibility |

**Prompt template for agents:**
```
I'm capturing an idea for the project. Please review for feasibility and refinement:

Idea: [brief description]

Questions:
1. Is this biologically/technically plausible?
2. What are the key trade-offs or gotchas?
3. Do you see any Golden Zone opportunities (optimization = emergent behavior)?
4. Suggested refinements?
```

Include the agent's key insights in the final idea document.

---

## Step 3: Golden Zone Discovery

**Before recording, actively look for Golden Zone opportunities.**

Golden Zone = performance optimization that ALSO creates emergent biological behavior for free.

Ask yourself:
- Can we skip work in a way that matches real biology?
- Does this optimization create interesting player-observable dynamics?
- Can one system serve two purposes (perf + gameplay)?

**Examples from this project:**
| Optimization | Free Biology |
|--------------|--------------|
| Skip perception of small entities | Size domination (giants ignore mice) |
| Skip stationary targets | Prey freeze = camouflage |
| Satiated creatures skip prey detection | Post-meal predators rest |
| FOV culling | Realistic blind spots |

If you find a Golden Zone angle, highlight it prominently in the idea.

---

## Step 4: Scan for Related Content

Search docs for related material:

1. **Existing ideas** - `docs/*/ideas/` - avoid duplication
2. **Implemented features** - `docs/*/done/` - what already exists
3. **Planned work** - `docs/*/todo/` and `ABC-SUPER_SPRINT/` - overlap check

Report:
- Similar ideas (merge or differentiate?)
- Related systems this would integrate with
- Potential conflicts or synergies

---

## Step 5: Draft and Save

### Pick the right location:

| Category | Path |
|----------|------|
| Creature traits, DNA, behavior, ecology | `docs/biology/ideas/` |
| Player experience, game mechanics, UI/UX | `docs/gameplay/ideas/` |
| Optimization, scaling, architecture | `docs/performance/ideas/` |
| Rendering, effects, aesthetics | `docs/visuals/ideas/` |
| Test infrastructure, validation | `docs/testing/ideas/` |

### Filename: kebab-case, descriptive (e.g., `pack-hunting.md`)

### Template:

```markdown
# [Idea Title]

## Problem / Opportunity

[What this addresses - keep abstract, no code references]

## Proposed Solution

[Core concept in 2-3 paragraphs - describe behavior/outcome, not implementation]

## Golden Zone

[If applicable: How does this optimization create emergent behavior?]
[If not applicable: "N/A - pure [category] feature"]

## Trade-offs

[What are the costs? What do we give up?]

## Expert Input

[Key insights from zoologist-tom / ecs-emma / etc., if consulted]

## Dependencies

- [What must exist first - reference concepts, not code]

## Related Ideas

- [Links to related docs in ideas/todo/done]

## Open Questions

- [Unresolved design decisions]

---
*Captured: [Date]*
```

### IMPORTANT: No Code References

Ideas must be **implementation-agnostic**:
- NO file paths (e.g., `perception/systems.rs`)
- NO code snippets
- NO function/struct names
- NO line numbers

**Why:** Code changes. Ideas are stable concepts. Implementation details come at sprint time.

**Instead of:** "Modify `calculate_avoidance_force()` in `steering/avoidance.rs`"
**Write:** "Modify the avoidance force calculation to account for closing velocity"

---

## Step 6: Confirm and Save

Show the user:
1. Proposed file path
2. Drafted content
3. Related items found
4. Expert insights (if consulted)

Ask for approval before writing. Allow edits.

---

## Step 7: Cross-Reference (Optional)

If the idea relates to existing docs, offer to add a "See also" link in those docs.

---

**Quick Category Reference:**
| Signal Words | Category |
|--------------|----------|
| creature, trait, gene, behavior, predator, prey, flee, hunt | biology |
| player, UI, controls, progression, sandbox, fun | gameplay |
| Hz, tick, cache, parallel, grid, latency, scaling | performance |
| shader, render, animation, effect, visual, overlay | visuals |
| test, validation, spec, coverage | testing |
