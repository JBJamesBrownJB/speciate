// Position-buffer layout for the Rust↔Electron seam (single source of truth on the JS
// side). These MUST match the Rust producer cap and layout in
// apps/simulation/src/ipc/bridge/double_buffer.rs (MAX_CREATURES) and export_positions
// (5 f32s per creature). If MAX_CREATURES here is below the Rust cap, the main-process
// receive buffer truncates positions silently at the seam.
//
// Known ceiling: ids cross as f32 (exact only to 2^24 ≈ 16.7M cumulative) — see
// docs/testing/bugs/f32-id-precision-ceiling.md.

const MAX_CREATURES = 1_000_000;
const FLOATS_PER_CREATURE = 5; // ID, X, Y, Rotation, Size

/** Float length for a creature receive buffer (SoA). Defaults to the full cap. */
function creatureBufferFloats(maxCreatures = MAX_CREATURES, floatsPerCreature = FLOATS_PER_CREATURE) {
  return maxCreatures * floatsPerCreature;
}

module.exports = { MAX_CREATURES, FLOATS_PER_CREATURE, creatureBufferFloats };
