/**
 * Manual trial loading integration test
 *
 * Tests loading trial templates via Electron → Rust IPC
 */

const { spawn } = require('child_process');
const msgpack = require('msgpack-lite');
const path = require('path');

const binaryPath = path.join(__dirname, '../simulation/target/debug/speciate');

console.log('🧪 Testing trial loading via IPC...\n');

// Spawn simulation
const simulation = spawn(binaryPath, [], {
  stdio: ['pipe', 'pipe', 'inherit'],
});

let buffer = Buffer.alloc(0);
let stateUpdateCount = 0;
let foundCreatures = false;

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

      // Check if trial creatures appeared (should be 100 from 10x10 grid + 4 default)
      if (state.creatures && state.creatures.length >= 100 && !foundCreatures) {
        foundCreatures = true;
        console.log('✅ Trial loaded successfully!');
        console.log(`   Creature count: ${state.creatures.length}`);
        console.log(`   Tick: ${state.tick}\n`);

        // Test passed
        setTimeout(() => {
          console.log(`✅ Test PASSED: Trial template loaded correctly`);
          simulation.kill();
          process.exit(0);
        }, 500);
      }
    } catch (err) {
      console.error('❌ Failed to decode state:', err);
    }
  }
});

// Wait for simulation to start, then send trial load command
setTimeout(() => {
  console.log('📤 Sending DevLoadTrial command...');

  const command = {
    type: 'dev_load_trial',
    template: 'default-spawn-baseline',
  };

  const payload = msgpack.encode(command);
  const length = Buffer.alloc(4);
  length.writeUInt32BE(payload.length, 0);

  simulation.stdin.write(length);
  simulation.stdin.write(payload);

  console.log(`   Template: ${command.template}`);
  console.log(`   Command size: ${payload.length} bytes`);
  console.log(`   Waiting for creatures to spawn...\n`);
}, 2000);

// Timeout if test takes too long
setTimeout(() => {
  if (!foundCreatures) {
    console.error('❌ Test FAILED: Trial did not load after 10 seconds');
    console.error(`   Received ${stateUpdateCount} state updates`);
    simulation.kill();
    process.exit(1);
  }
}, 10000);

simulation.on('exit', (code) => {
  if (code !== 0 && code !== null && !foundCreatures) {
    console.error(`❌ Simulation exited with code ${code}`);
    process.exit(1);
  }
});
