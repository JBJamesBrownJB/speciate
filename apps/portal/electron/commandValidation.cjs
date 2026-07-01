/**
 * Renderer → main command validation (whitelist + parameter checks).
 *
 * Salvaged from the retired stdio path and adapted to the NAPI command set.
 * The renderer is semi-trusted (contextIsolation on, but any renderer bug
 * becomes main-process input); `template` in particular reaches Rust file
 * loading, so it must never carry path separators. Throws on invalid input —
 * callers catch and drop the command.
 */

// Generous world-coordinate bound (engine world is smaller; this only guards
// against garbage like 1e300 or NaN reaching Rust).
const COORD_LIMIT = 1_000_000;

function assertFiniteNumber(value, label) {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw new Error(`${label} must be a finite number`);
  }
}

function assertOptionalGene(dna, key) {
  if (dna === undefined || dna === null) return;
  if (typeof dna !== 'object') throw new Error('dna must be an object');
  const gene = dna[key];
  if (gene === undefined || gene === null) return;
  assertFiniteNumber(gene, `dna.${key}`);
}

const COMMAND_VALIDATORS = {
  dev_spawn_creature: (command) => {
    assertFiniteNumber(command.x, 'dev_spawn_creature: x');
    assertFiniteNumber(command.y, 'dev_spawn_creature: y');
    if (Math.abs(command.x) > COORD_LIMIT || Math.abs(command.y) > COORD_LIMIT) {
      throw new Error('dev_spawn_creature: coordinates out of world bounds');
    }
    assertOptionalGene(command.dna, 'size_gene');
    assertOptionalGene(command.dna, 'fov_gene');
  },

  dev_load_trial: (command) => {
    if (typeof command.template !== 'string' || command.template.length === 0) {
      throw new Error('dev_load_trial: template must be a non-empty string');
    }
    // Path traversal prevention — the template name reaches Rust file loading.
    if (/[/\\]|\.\./.test(command.template)) {
      throw new Error('dev_load_trial: template name contains invalid characters');
    }
    assertOptionalGene(command.dna, 'size_gene');
    assertOptionalGene(command.dna, 'fov_gene');
  },

  dev_clear_creatures: () => {},

  dev_clear_plants: () => {},
};

/** Validate a send-command payload. Throws with a reason on any violation. */
function validateCommand(command) {
  if (typeof command !== 'object' || command === null) {
    throw new Error('command must be an object');
  }
  const validator = COMMAND_VALIDATORS[command.type];
  if (!validator) {
    throw new Error(`unknown command type: ${command.type}`);
  }
  validator(command);
}

module.exports = { validateCommand };
