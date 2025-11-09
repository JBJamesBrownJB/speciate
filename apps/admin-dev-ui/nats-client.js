/**
 * NATS Client for Dev Admin
 *
 * Connects to NATS server via WebSocket and publishes dev commands.
 * Uses nats.ws library loaded from CDN.
 */

import {
  connect,
  StringCodec,
} from "https://cdn.jsdelivr.net/npm/nats.ws@1.28.0/+esm";

const NATS_URL = "ws://localhost:9224";
const DEV_COMMAND_SUBJECT_PREFIX = "dev.sim";

let natsClient = null;
let statusCallback = null;

const sc = StringCodec();

/**
 * Connect to NATS server
 * @param {Function} onStatusChange - Callback for connection status changes
 * @returns {Promise<void>}
 */
export async function connectToNATS(onStatusChange) {
  statusCallback = onStatusChange;

  try {
    updateStatus("connecting");
    console.log("[NATS] Connecting to", NATS_URL);

    natsClient = await connect({
      servers: NATS_URL,
    });

    console.log("[NATS] Connected successfully");
    updateStatus("connected");

    // Handle connection close
    (async () => {
      for await (const status of natsClient.status()) {
        console.log("[NATS] Status:", status);
        if (status.type === "disconnect" || status.type === "error") {
          updateStatus("disconnected");
        }
      }
    })();
  } catch (error) {
    console.error("[NATS] Connection failed:", error);
    updateStatus("disconnected");
    throw error;
  }
}

/**
 * Publish a dev command to NATS
 * @param {string} commandType - Command type (spawn, clear, speed)
 * @param {Object} data - Command data
 * @returns {Promise<void>}
 */
export async function publishDevCommand(commandType, data) {
  if (!natsClient) {
    throw new Error("Not connected to NATS");
  }

  const subject = `${DEV_COMMAND_SUBJECT_PREFIX}.${commandType}`;
  const payload = JSON.stringify(data);

  console.log("[NATS] Publishing to", subject, ":", data);

  try {
    natsClient.publish(subject, sc.encode(payload));
    await natsClient.flush(); // Ensure message is sent
    console.log("[NATS] Published successfully");
  } catch (error) {
    console.error("[NATS] Publish failed:", error);
    throw error;
  }
}

/**
 * Publish a spawn command
 * @param {Object} spawnData - Spawn parameters
 * @returns {Promise<void>}
 */
export async function publishSpawn(spawnData) {
  return publishDevCommand("spawn", spawnData);
}

/**
 * Publish a clear command
 * @returns {Promise<void>}
 */
export async function publishClear() {
  return publishDevCommand("clear", { type: "Clear" });
}

/**
 * Publish a speed command
 * @param {number} multiplier - Speed multiplier (0.25 - 5.0)
 * @returns {Promise<void>}
 */
export async function publishSpeed(multiplier) {
  return publishDevCommand("speed", {
    type: "Speed",
    multiplier: multiplier,
  });
}

/**
 * Update connection status
 * @param {string} status - Status: 'connected', 'disconnected', 'connecting'
 */
function updateStatus(status) {
  if (statusCallback) {
    statusCallback(status);
  }
}

/**
 * Close NATS connection
 * @returns {Promise<void>}
 */
export async function disconnect() {
  if (natsClient) {
    await natsClient.close();
    natsClient = null;
    updateStatus("disconnected");
  }
}
