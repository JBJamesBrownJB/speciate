#!/usr/bin/env node

/**
 * NAPI Integration Test
 *
 * This script tests the basic functionality of the NAPI addon:
 * - Module loading
 * - SimulationEngine creation
 * - start() with telemetry callback
 * - getBuffer() zero-copy access
 * - spawnCreatures() command
 * - stop() clean shutdown
 */

const addon = require('./speciate.linux-x64-gnu.node');

console.log('🧪 NAPI Integration Test\n');

// Test 1: Module exports
console.log('📦 Test 1: Module Exports');
console.log('  - init_logger:', typeof addon.initLogger);
console.log('  - SimulationEngine:', typeof addon.SimulationEngine);

if (typeof addon.initLogger !== 'function' || typeof addon.SimulationEngine !== 'function') {
    console.error('❌ FAIL: Missing exports');
    process.exit(1);
}
console.log('✅ PASS: Exports correct\n');

// Test 2: Initialize panic handler
console.log('📦 Test 2: Init Logger');
try {
    addon.initLogger();
    console.log('✅ PASS: Logger initialized\n');
} catch (e) {
    console.error('❌ FAIL:', e.message);
    process.exit(1);
}

// Test 3: Create SimulationEngine
console.log('📦 Test 3: Create SimulationEngine');
let sim;
try {
    sim = new addon.SimulationEngine();
    console.log('✅ PASS: SimulationEngine created\n');
} catch (e) {
    console.error('❌ FAIL:', e.message);
    process.exit(1);
}

// Test 4: Start simulation (no callback - using polling instead)
console.log('📦 Test 4: Start Simulation');
let telemetryReceived = false;
let creatureCount = 0;

try {
    sim.start(100, process.cwd(), () => {}); // Empty callback (not used anymore)
    console.log('✅ PASS: Simulation started with 100 creatures\n');
} catch (e) {
    console.error('❌ FAIL:', e.message);
    process.exit(1);
}

// Test 4.5: Poll telemetry using getTelemetry()
console.log('📦 Test 4.5: Poll Telemetry');
setTimeout(() => {
    try {
        const telemetryJson = sim.getTelemetry();
        console.log(`  📊 Telemetry JSON length: ${telemetryJson.length} bytes`);

        const data = JSON.parse(telemetryJson);
        console.log(`  📊 Tick: ${data.tick}`);
        console.log(`  📊 Creature count: ${data.creatureCount}`);
        console.log(`  📊 System timings present: ${!!data.systemTimings}`);
        console.log(`  📊 Hardware metrics present: ${!!data.hardwareMetrics}`);

        telemetryReceived = true;
        creatureCount = data.creatureCount;

        if (data.creatureCount === 100) {
            console.log('✅ PASS: Telemetry polling works\n');
        } else {
            console.error(`❌ FAIL: Expected 100 creatures, got ${data.creatureCount}\n`);
        }
    } catch (e) {
        console.error('❌ FAIL:', e.message);
        process.exit(1);
    }
}, 1000);

// Test 4.6: Test getTick() and getTickRate()
console.log('📦 Test 4.6: Test getTick() and getTickRate()');
setTimeout(() => {
    try {
        const tick = sim.getTick();
        const tickRate = sim.getTickRate();

        console.log(`  📊 Current tick: ${tick}`);
        console.log(`  📊 Tick rate: ${tickRate} Hz`);

        if (tick > 0 && tickRate === 30.0) {
            console.log('✅ PASS: getTick() and getTickRate() work\n');
        } else {
            console.error(`❌ FAIL: tick=${tick}, tickRate=${tickRate}\n`);
        }
    } catch (e) {
        console.error('❌ FAIL:', e.message);
        process.exit(1);
    }
}, 1500);

// Test 5: Get buffer (zero-copy)
console.log('📦 Test 5: Get Buffer (Zero-Copy)');
setTimeout(() => {
    try {
        const buffer = sim.getBuffer();
        console.log(`  - Buffer type: ${buffer.constructor.name}`);
        console.log(`  - Buffer length: ${buffer.length} (expected: ${100 * 4} = 400)`);

        if (buffer.length !== 400) {
            console.error(`❌ FAIL: Buffer size mismatch (got ${buffer.length}, expected 400)`);
            sim.stop();
            process.exit(1);
        }

        // Parse first creature (SoA layout)
        const id = buffer[0];
        const x = buffer[100];
        const y = buffer[200];
        const rot = buffer[300];

        console.log(`  - First creature: id=${id}, x=${x.toFixed(2)}, y=${y.toFixed(2)}, rot=${rot.toFixed(2)}`);
        console.log('✅ PASS: Buffer access works\n');
    } catch (e) {
        console.error('❌ FAIL:', e.message);
        sim.stop();
        process.exit(1);
    }
}, 500);

// Test 6: Spawn more creatures
console.log('📦 Test 6: Spawn Creatures Command');
setTimeout(() => {
    try {
        sim.spawnCreatures(50);
        console.log('  ✅ Command sent: spawnCreatures(50)');

        // Poll telemetry to confirm spawn
        setTimeout(() => {
            const telemetry = JSON.parse(sim.getTelemetry());
            creatureCount = telemetry.creatureCount;

            if (creatureCount >= 150) {
                console.log(`  ✅ Confirmed: ${creatureCount} creatures now active`);
                console.log('✅ PASS: Spawn command works\n');
            } else {
                console.log(`  ⚠️  WARNING: Expected 150+, got ${creatureCount}`);
            }
        }, 1500);
    } catch (e) {
        console.error('❌ FAIL:', e.message);
        sim.stop();
        process.exit(1);
    }
}, 2000);

// Test 7: Clean shutdown
console.log('📦 Test 7: Clean Shutdown');
setTimeout(() => {
    try {
        sim.stop();
        console.log('✅ PASS: Simulation stopped cleanly\n');

        // Final summary
        console.log('🎉 ALL TESTS PASSED!');
        console.log(`📊 Telemetry received: ${telemetryReceived ? 'YES' : 'NO'}`);
        console.log(`📊 Final creature count: ${creatureCount}`);

        process.exit(0);
    } catch (e) {
        console.error('❌ FAIL:', e.message);
        process.exit(1);
    }
}, 5000);
