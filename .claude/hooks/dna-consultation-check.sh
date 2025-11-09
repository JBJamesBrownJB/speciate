#!/bin/bash
# PreToolUse hook: DNA-Driven Design Guidance
#
# This hook provides warnings and guidance when editing creature-related code,
# reminding developers to:
# 1. Encode all traits in DNA (not hardcoded)
# 2. Consult zoologist-tom for biological boundaries
# 3. Log decisions in /docs/biology/biology-notes.md
#
# MODE: Warning + Guidance (non-blocking)

# Read the tool use JSON from stdin
TOOL_USE=$(cat)

# Extract tool name and file path
TOOL_NAME=$(echo "$TOOL_USE" | jq -r '.tool // empty')
FILE_PATH=$(echo "$TOOL_USE" | jq -r '.parameters.file_path // empty')

# Only check Edit and Write operations
if [[ "$TOOL_NAME" != "Edit" && "$TOOL_NAME" != "Write" ]]; then
  exit 0  # Allow other tools
fi

# Define creature-related files that should trigger guidance
CREATURE_FILES=(
  "apps/simulation/src/simulation/components.rs"
  "apps/simulation/src/spawner.rs"
  "apps/simulation/src/simulation/crit_systems.rs"
  "apps/simulation/src/simulation/movement.rs"
  "apps/simulation/src/simulation/perception.rs"
  "apps/simulation/src/simulation/behavior.rs"
  "apps/simulation/src/simulation/genetics.rs"
  "docs/biology/dna-driven-design.md"
  "docs/biology/biology-notes.md"
)

# Check if the file path matches any creature-related files
SHOULD_WARN=false
for pattern in "${CREATURE_FILES[@]}"; do
  if [[ "$FILE_PATH" == *"$pattern"* ]]; then
    SHOULD_WARN=true
    break
  fi
done

# Skip guidance if not a creature-related file
if [[ "$SHOULD_WARN" == "false" ]]; then
  exit 0
fi

# Display DNA-driven design guidance
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "🧬  DNA-DRIVEN DESIGN REMINDER" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "" >&2
echo "You're editing creature-related code: $FILE_PATH" >&2
echo "" >&2

# Extract the actual code change to provide context-specific guidance
NEW_STRING=$(echo "$TOOL_USE" | jq -r '.parameters.new_string // empty')
OLD_STRING=$(echo "$TOOL_USE" | jq -r '.parameters.old_string // empty')
CONTENT=$(echo "$TOOL_USE" | jq -r '.parameters.content // empty')

# Check for new struct fields (common pattern: "pub field_name: Type")
if echo "$NEW_STRING$CONTENT" | grep -qE "pub [a-z_]+: (f32|f64|i32|u32|bool|String)"; then
  echo "⚠️  DETECTED: New struct field(s)" >&2
  echo "" >&2
  echo "   Before adding creature attributes, ask yourself:" >&2
  echo "   ✓ Should this trait be DNA-encoded for genetic variation?" >&2
  echo "   ✓ Have I consulted zoologist-tom for realistic min/max bounds?" >&2
  echo "   ✓ Does this enable emergent behavior or just hardcode values?" >&2
  echo "" >&2
fi

# Check for hardcoded constants (potential DNA candidates)
if echo "$NEW_STRING$CONTENT" | grep -qE "(const|static) [A-Z_]+ ?: ?(f32|f64|i32)"; then
  echo "⚠️  DETECTED: Hardcoded constant(s)" >&2
  echo "" >&2
  echo "   Constants prevent genetic diversity!" >&2
  echo "   ✓ Could this be a DNA gene instead of a global constant?" >&2
  echo "   ✓ Would creatures benefit from variation in this value?" >&2
  echo "" >&2
fi

# Check if editing components.rs specifically
if [[ "$FILE_PATH" == *"components.rs"* ]]; then
  echo "📋 COMPONENTS.RS GUIDANCE:" >&2
  echo "   • All creature traits should be in the Dna component" >&2
  echo "   • Avoid adding fields to CreatureState unless runtime-only" >&2
  echo "   • Physical/behavioral traits → DNA genes" >&2
  echo "   • Temporary state (current energy) → CreatureState" >&2
  echo "" >&2
fi

# Check if editing spawner.rs
if [[ "$FILE_PATH" == *"spawner.rs"* ]]; then
  echo "📋 SPAWNER.RS GUIDANCE:" >&2
  echo "   • New creatures should spawn with randomized DNA" >&2
  echo "   • Use DNA to drive initial CreatureState values" >&2
  echo "   • Avoid hardcoded spawn parameters" >&2
  echo "" >&2
fi

# Core reminders (always show)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "🔬  REQUIRED STEPS:" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "" >&2
echo "1. 📖 Read the design doc (if you haven't):" >&2
echo "   /workspace/docs/biology/dna-driven-design.md" >&2
echo "" >&2
echo "2. 🦎 Consult zoologist-tom for trait boundaries:" >&2
echo "   Use: Task tool with subagent_type: zoologist-tom" >&2
echo "   Ask: \"What's a realistic range for [trait]?\"" >&2
echo "   Ask: \"How should [trait] scale with size/energy?\"" >&2
echo "" >&2
echo "3. 📝 Log the decision in /docs/biology/biology-notes.md:" >&2
echo "   Format: Date | Feature | Zoologist Input | Implementation" >&2
echo "   This creates permanent record for future reference" >&2
echo "" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2

# Check if editing behavior/decision logic
if echo "$NEW_STRING$CONTENT" | grep -qE "(if|match|when|should|decide)"; then
  echo "💡 TIP: Behavior decisions should use DNA traits" >&2
  echo "" >&2
  echo "   Example:" >&2
  echo "   ✗ BAD:  if distance < 5.0 { avoid(); }  // Hardcoded" >&2
  echo "   ✓ GOOD: if distance < dna.personal_space { avoid(); }" >&2
  echo "" >&2
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
fi

# Detect legacy hardcoded traits (technical debt flags)
LEGACY_PATTERNS=(
  "max_speed"
  "max_energy"
  "perception_range"
  "vision_distance"
  "avoidance_threshold"
)

for pattern in "${LEGACY_PATTERNS[@]}"; do
  if echo "$OLD_STRING$NEW_STRING$CONTENT" | grep -q "$pattern"; then
    echo "🏗️  TECHNICAL DEBT DETECTED: $pattern" >&2
    echo "" >&2
    echo "   This trait is currently hardcoded but should be DNA-encoded." >&2
    echo "   If you're working on this, great! If not, note for future migration." >&2
    echo "" >&2
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
    break
  fi
done

# Special guidance for DNA documentation edits
if [[ "$FILE_PATH" == *"dna-driven-design.md"* ]] || [[ "$FILE_PATH" == *"biology-notes.md"* ]]; then
  echo "📚 DOCUMENTATION UPDATE DETECTED" >&2
  echo "" >&2
  echo "   Great! Keeping docs updated is crucial." >&2
  echo "   Remember to:" >&2
  echo "   ✓ Update both design doc AND biology notes if adding traits" >&2
  echo "   ✓ Include zoologist rationale for any bounds/formulas" >&2
  echo "   ✓ Link to implementation files" >&2
  echo "" >&2
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
fi

echo "" >&2
echo "🎯 GOAL: Every creature trait encoded in DNA for emergent ecosystems" >&2
echo "" >&2
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" >&2
echo "" >&2

# Always exit 0 (non-blocking, guidance only)
exit 0
