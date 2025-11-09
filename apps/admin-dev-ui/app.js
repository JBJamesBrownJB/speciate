/**
 * Admin UI Application
 *
 * Main application logic for the dev admin control panel.
 */

import { connectToNATS, publishSpawn, publishClear, publishSpeed } from './nats-client.js';
import { getScenario } from './scenarios.js';

// DOM Elements
const statusElement = document.getElementById('status');
const scenarioButtons = document.querySelectorAll('.scenario-btn');
const manualSpawnForm = document.getElementById('manualSpawnForm');
const behaviorSelect = document.getElementById('behavior');
const targetGroup = document.getElementById('targetGroup');
const clearAllBtn = document.getElementById('clearAll');
const speedSlider = document.getElementById('speedSlider');
const speedValue = document.getElementById('speedValue');
const speedPresets = document.querySelectorAll('.speed-preset');
const logElement = document.getElementById('log');

// Application state
let isConnected = false;

/**
 * Initialize application
 */
async function init() {
    log('Initializing admin UI...', 'info');

    // Connect to NATS
    try {
        await connectToNATS(onStatusChange);
        log('Connected to NATS successfully', 'success');
    } catch (error) {
        log(`Failed to connect to NATS: ${error.message}`, 'error');
    }

    // Setup event listeners
    setupEventListeners();

    log('Admin UI ready', 'success');
}

/**
 * Setup event listeners
 */
function setupEventListeners() {
    // Scenario buttons
    scenarioButtons.forEach(btn => {
        btn.addEventListener('click', () => {
            const scenarioName = btn.dataset.scenario;
            executeScenario(scenarioName);
        });
    });

    // Manual spawn form
    manualSpawnForm.addEventListener('submit', (e) => {
        e.preventDefault();
        executeManualSpawn();
    });

    // Behavior select - show/hide target fields
    behaviorSelect.addEventListener('change', () => {
        const isSeeking = behaviorSelect.value === 'seeking';
        targetGroup.style.display = isSeeking ? 'grid' : 'none';
    });

    // Clear all button
    clearAllBtn.addEventListener('click', async () => {
        if (!isConnected) {
            log('Not connected to NATS', 'error');
            return;
        }

        try {
            log('Clearing all creatures...', 'info');
            await publishClear();
            log('Clear command sent', 'success');
        } catch (error) {
            log(`Clear failed: ${error.message}`, 'error');
        }
    });

    // Speed slider
    speedSlider.addEventListener('input', (e) => {
        const multiplier = parseFloat(e.target.value);
        speedValue.textContent = `${multiplier.toFixed(2)}x`;
    });

    speedSlider.addEventListener('change', async (e) => {
        const multiplier = parseFloat(e.target.value);
        await setSpeed(multiplier);
    });

    // Speed presets
    speedPresets.forEach(btn => {
        btn.addEventListener('click', async () => {
            const speed = parseFloat(btn.dataset.speed);
            speedSlider.value = speed;
            speedValue.textContent = `${speed.toFixed(2)}x`;
            await setSpeed(speed);
        });
    });
}

/**
 * Execute a scenario
 * @param {string} scenarioName - Name of the scenario
 */
async function executeScenario(scenarioName) {
    if (!isConnected) {
        log('Not connected to NATS', 'error');
        return;
    }

    try {
        log(`Executing scenario: ${scenarioName}`, 'info');

        const commands = getScenario(scenarioName);
        log(`Spawning ${commands.length} creatures...`, 'info');

        for (const command of commands) {
            await publishSpawn(command);
        }

        log(`Scenario "${scenarioName}" executed successfully (${commands.length} spawns)`, 'success');
    } catch (error) {
        log(`Scenario failed: ${error.message}`, 'error');
    }
}

/**
 * Execute manual spawn from form
 */
async function executeManualSpawn() {
    if (!isConnected) {
        log('Not connected to NATS', 'error');
        return;
    }

    try {
        const x = parseFloat(document.getElementById('posX').value);
        const y = parseFloat(document.getElementById('posY').value);
        const behavior = behaviorSelect.value;

        const command = {
            type: 'Spawn',
            x: x,
            y: y,
            behavior: behavior
        };

        // Add target for seeking behavior
        if (behavior === 'seeking') {
            command.target_x = parseFloat(document.getElementById('targetX').value);
            command.target_y = parseFloat(document.getElementById('targetY').value);
        }

        // Add optional parameters if provided
        const energy = document.getElementById('energy').value;
        if (energy) {
            command.energy = parseFloat(energy);
        }

        const maxSpeed = document.getElementById('maxSpeed').value;
        if (maxSpeed) {
            command.max_speed = parseFloat(maxSpeed);
        }

        log(`Spawning ${behavior} creature at (${x}, ${y})`, 'info');
        await publishSpawn(command);
        log('Creature spawned successfully', 'success');
    } catch (error) {
        log(`Spawn failed: ${error.message}`, 'error');
    }
}

/**
 * Set simulation speed
 * @param {number} multiplier - Speed multiplier
 */
async function setSpeed(multiplier) {
    if (!isConnected) {
        log('Not connected to NATS', 'error');
        return;
    }

    try {
        log(`Setting speed to ${multiplier.toFixed(2)}x`, 'info');
        await publishSpeed(multiplier);
        log(`Speed set to ${multiplier.toFixed(2)}x`, 'success');
    } catch (error) {
        log(`Speed change failed: ${error.message}`, 'error');
    }
}

/**
 * Handle NATS connection status change
 * @param {string} status - Connection status
 */
function onStatusChange(status) {
    statusElement.className = `status ${status}`;

    switch (status) {
        case 'connected':
            statusElement.textContent = 'Connected';
            isConnected = true;
            break;
        case 'disconnected':
            statusElement.textContent = 'Disconnected';
            isConnected = false;
            break;
        case 'connecting':
            statusElement.textContent = 'Connecting...';
            isConnected = false;
            break;
    }
}

/**
 * Log message to activity log
 * @param {string} message - Log message
 * @param {string} type - Log type: 'info', 'success', 'error'
 */
function log(message, type = 'info') {
    const timestamp = new Date().toLocaleTimeString();
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.innerHTML = `
        <span class="timestamp">[${timestamp}]</span>
        <span class="message">${message}</span>
    `;

    logElement.insertBefore(entry, logElement.firstChild);

    // Keep only last 50 entries
    while (logElement.children.length > 50) {
        logElement.removeChild(logElement.lastChild);
    }

    // Also log to console
    console.log(`[${timestamp}] ${message}`);
}

// Initialize on page load
init();
