/**
 * Scenario Templates
 *
 * Defines test scenarios for rapid visual testing of creature behaviors.
 * Each scenario returns an array of spawn commands.
 */

export const scenarios = {
    /**
     * Two Seekers Intercept
     *
     * Tests collision avoidance when two creatures approach each other head-on.
     * They should detect each other and avoid collision while still seeking.
     */
    twoSeekersIntercept: () => {
        return [
            {
                type: "Spawn",
                x: -100.0,
                y: 0.0,
                behavior: "seeking",
                target_x: 100.0,
                target_y: 0.0,
                energy: 100.0,
                max_speed: 20.0
            },
            {
                type: "Spawn",
                x: 100.0,
                y: 0.0,
                behavior: "seeking",
                target_x: -100.0,
                target_y: 0.0,
                energy: 100.0,
                max_speed: 20.0
            }
        ];
    },

    /**
     * Wanderer Crowd
     *
     * Tests personal space and flocking behavior with multiple wanderers
     * spawned in close proximity.
     */
    wandererCrowd: () => {
        const commands = [];
        const centerX = 0;
        const centerY = 0;
        const radius = 10; // Spawn in 10m radius
        const count = 15;

        for (let i = 0; i < count; i++) {
            const angle = (i / count) * Math.PI * 2;
            const distance = Math.random() * radius;
            const x = centerX + Math.cos(angle) * distance;
            const y = centerY + Math.sin(angle) * distance;

            commands.push({
                type: "Spawn",
                x: x,
                y: y,
                behavior: "wandering",
                energy: 100.0,
                max_speed: 15.0
            });
        }

        return commands;
    },

    /**
     * Seeker + Obstacles
     *
     * Tests obstacle avoidance while seeking a target.
     * One seeker navigates through a field of catatonic creatures (obstacles).
     */
    seekerObstacles: () => {
        const commands = [];

        // Spawn seeker at left
        commands.push({
            type: "Spawn",
            x: -80.0,
            y: 0.0,
            behavior: "seeking",
            target_x: 80.0,
            target_y: 0.0,
            energy: 100.0,
            max_speed: 25.0
        });

        // Spawn obstacles in a grid pattern
        for (let x = -40; x <= 40; x += 20) {
            for (let y = -30; y <= 30; y += 20) {
                // Skip center area to give seeker a path
                if (Math.abs(x) < 15 && Math.abs(y) < 15) continue;

                commands.push({
                    type: "Spawn",
                    x: x,
                    y: y,
                    behavior: "catatonic",
                    energy: 100.0,
                    max_speed: 0.0
                });
            }
        }

        return commands;
    },

    /**
     * Ring of Death
     *
     * Stress test for multi-body collision resolution.
     * 8 creatures arranged in a circle, all seeking the center point.
     */
    ringOfDeath: () => {
        const commands = [];
        const centerX = 0;
        const centerY = 0;
        const radius = 50; // Start 50m from center
        const count = 8;

        for (let i = 0; i < count; i++) {
            const angle = (i / count) * Math.PI * 2;
            const x = centerX + Math.cos(angle) * radius;
            const y = centerY + Math.sin(angle) * radius;

            commands.push({
                type: "Spawn",
                x: x,
                y: y,
                behavior: "seeking",
                target_x: centerX,
                target_y: centerY,
                energy: 100.0,
                max_speed: 30.0
            });
        }

        return commands;
    }
};

/**
 * Get scenario by name
 * @param {string} name - Scenario name
 * @returns {Array} Array of spawn commands
 */
export function getScenario(name) {
    const scenario = scenarios[name];
    if (!scenario) {
        throw new Error(`Unknown scenario: ${name}`);
    }
    return scenario();
}
