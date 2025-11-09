#!/bin/bash
# Network Traffic Monitor for Speciate
# Monitors traffic between Sim -> NATS -> Broadcaster -> Portal

set -euo pipefail

# Trap Ctrl+C for graceful shutdown
trap 'echo ""; echo "Monitoring stopped."; exit 0' INT

INTERVAL=${1:-10}
SAMPLES=${2:-0}  # 0 = infinite

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║          Speciate Network Traffic Monitor                      ║"
echo "╟────────────────────────────────────────────────────────────────╢"
echo "║  Monitoring: Sim -> NATS -> Broadcaster -> Portal              ║"
if [ "$SAMPLES" -eq 0 ]; then
    echo "║  Interval: ${INTERVAL}s | Mode: Continuous (Ctrl+C to stop)           ║"
else
    echo "║  Interval: ${INTERVAL}s | Samples: ${SAMPLES}                              ║"
fi
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Function to parse docker stats NetIO format (e.g., "1.5MB / 2.3MB")
parse_netio() {
    local netio=$1
    local direction=$2  # "rx" or "tx"

    if [ "$direction" = "rx" ]; then
        echo "$netio" | awk -F' / ' '{print $1}'
    else
        echo "$netio" | awk -F' / ' '{print $2}'
    fi
}

# Convert human-readable sizes to bytes
to_bytes() {
    local size=$1
    python3 -c "
import re
size = '$size'
match = re.match(r'([0-9.]+)\s*([A-Za-z]*)', size)
if not match:
    print(0)
else:
    num = float(match.group(1))
    unit = match.group(2).upper()
    multipliers = {'B': 1, 'KB': 1000, 'MB': 1000000, 'GB': 1000000000}
    print(int(num * multipliers.get(unit, 1)))
"
}

# Format bytes to human readable
format_bytes() {
    local bytes=$1
    python3 -c "
bytes = $bytes
if bytes < 1000:
    print(f'{bytes:.1f} B')
elif bytes < 1000000:
    print(f'{bytes/1000:.1f} kB')
elif bytes < 1000000000:
    print(f'{bytes/1000000:.2f} MB')
else:
    print(f'{bytes/1000000000:.2f} GB')
"
}

# Get initial stats
get_stats() {
    local container=$1
    if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
        docker stats --no-stream --format "{{.NetIO}}" "$container" 2>/dev/null || echo "0B / 0B"
    else
        echo "0B / 0B"
    fi
}

echo "Timestamp                  | NATS RX Rate | NATS TX Rate | Connections"
echo "───────────────────────────┼──────────────┼──────────────┼────────────"

# Main monitoring loop
COUNTER=0
while true; do
    # Get initial values
    NATS_STATS=$(get_stats "speciate-nats")
    NATS_RX1=$(to_bytes "$(parse_netio "$NATS_STATS" "rx")")
    NATS_TX1=$(to_bytes "$(parse_netio "$NATS_STATS" "tx")")

    sleep $INTERVAL

    # Get values after interval
    NATS_STATS=$(get_stats "speciate-nats")
    NATS_RX2=$(to_bytes "$(parse_netio "$NATS_STATS" "rx")")
    NATS_TX2=$(to_bytes "$(parse_netio "$NATS_STATS" "tx")")

    # Calculate rates
    RX_BYTES=$(python3 -c "print(int($NATS_RX2 - $NATS_RX1))")
    TX_BYTES=$(python3 -c "print(int($NATS_TX2 - $NATS_TX1))")
    RX_RATE=$(python3 -c "print(int($RX_BYTES / $INTERVAL))")
    TX_RATE=$(python3 -c "print(int($TX_BYTES / $INTERVAL))")

    # Count connections
    CONN_COUNT=$(ss -tunp 2>/dev/null | grep -cE '(4222|8080|8081)' || echo "0")

    # Format output
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')
    printf "%-27s| %12s | %12s | %d\n" \
        "$TIMESTAMP" \
        "$(format_bytes $RX_RATE)/s" \
        "$(format_bytes $TX_RATE)/s" \
        "$CONN_COUNT"

    # Check if we should stop (if SAMPLES is set and not 0)
    COUNTER=$((COUNTER + 1))
    if [ "$SAMPLES" -gt 0 ] && [ "$COUNTER" -ge "$SAMPLES" ]; then
        break
    fi
done

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  Connection Details                                            ║"
echo "╚════════════════════════════════════════════════════════════════╝"
ss -tunp 2>/dev/null | grep -E '(4222|8080|8081)' | \
    awk '{printf "%-8s %20s -> %-20s %s\n", $1, $5, $6, $7}' || \
    echo "No active connections"

echo ""
echo "Legend:"
echo "  - NATS RX: Data received by NATS (from Sim)"
echo "  - NATS TX: Data sent by NATS (to Broadcaster)"
echo "  - Port 4222: NATS client connections (Sim, Broadcaster)"
echo "  - Port 8080/8081: WebSocket connections (Broadcaster -> Portal)"
