/**
 * Manual IPC integration test
 *
 * Tests the full Electron → Rust communication:
 * 1. Spawn Rust simulation binary
 * 2. Send DevSpawnCreature command via stdin (MessagePack)
 * 3. Verify simulation receives stdout state updates
 * 4. Verify creature appears in state
 */

const { spawn } = require('child_process');
const msgpack = require('msgpack-lite');
const path = require('path');

const binaryPath = path.join(__dirname, '../simulation/target/debug/speciate');

console.log('🧪 Starting IPC integration test...\n');

// Spawn simulation
const simulation = spawn(binaryPath, [], {
  stdio: ['pipe', 'pipe', 'inherit'], // stdin, stdout, stderr
});

let buffer = Buffer.alloc(0);
let stateUpdateCount = 0;
let foundCreature = false;

// Read stdout frames
simulation.stdout.on('data', (chunk) => {
  buffer = Buffer.concat([buffer, chunk]);

  while (buffer.length >= 4) {
    const frameLength = buffer.readUInt32BE(0);
    if (buffer.length < 4 + frameLength) break;

    const payload = buffer.slice(4, 4 + frameLength);
    buffer = buffer.slice(4 + frameLength);

    try {
      const state = msgpack.decode(payload);
      stateUpdateCount++;

      // Check if our spawned creature (at 100, 200) appeared
      if (state.creatures && state.creatures.length > 0) {
        const creature = state.creatures.find(c =>
          Math.abs(c.x - 100) < 0.1 && Math.abs(c.y - 200) < 0.1
        );
        if (creature && !foundCreature) {
          foundCreature = true;
          console.log('✅ Found spawned creature at (100, 200)!');
          console.log(`   Creature ID: ${creature.id}`);
          console.log(`   Position: (${creature.x}, ${creature.y})`);
          console.log(`   Tick: ${state.tick}\n`);

          // Test passed - kill simulation
          setTimeout(() => {
            console.log(`✅ Test PASSED: Received ${stateUpdateCount} state updates`);
            console.log('✅ Creature spawned successfully via stdin command\n');
            simulation.kill();
            process.exit(0);
          }, 500);
        }
      }
    } catch (err) {
      console.error('❌ Failed to decode state:', err);
    }
  }
});

// Wait for simulation to start, then send spawn command
setTimeout(() => {
  console.log('📤 Sending DevSpawnCreature command...');

  const command = {
    type: 'dev_spawn_creature',
    x: 100.0,
    y: 200.0,
    dna: null,
  };

  const payload = msgpack.encode(command);
  const length = Buffer.alloc(4);
  length.writeUInt32BE(payload.length, 0);

  simulation.stdin.write(length);
  simulation.stdin.write(payload);

  console.log(`   Command size: ${payload.length} bytes`);
  console.log(`   Waiting for state updates...\n`);
}, 2000);

// Timeout if test takes too long
setTimeout(() => {
  if (!foundCreature) {
    console.error('❌ Test FAILED: Creature did not appear in state after 10 seconds');
    console.error(`   Received ${stateUpdateCount} state updates`);
    simulation.kill();
    process.exit(1);
  }
}, 10000);

simulation.on('exit', (code) => {
  if (code !== 0 && code !== null && !foundCreature) {
    console.error(`❌ Simulation exited with code ${code}`);
    process.exit(1);
  }
});
