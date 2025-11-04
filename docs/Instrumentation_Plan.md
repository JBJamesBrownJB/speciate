# Instrumentation & Observability Plan
## Performance Monitoring for Speciate Simulation Platform

**Date:** 2025-11-04
**Sprint:** 5 - Performance Instrumentation
**Status:** Planning Document
**Owner:** Backend Simulation Team + DevOps

---

## Executive Summary

This document defines the comprehensive observability and instrumentation strategy for the Speciate platform, with immediate focus on the new streaming architecture but applicable to all system components.

**Goals:**
1. **Visibility** - Real-time insight into system performance
2. **Debugging** - Rapid identification of bottlenecks and failures
3. **Capacity Planning** - Data-driven scaling decisions
4. **User Experience** - Ensure responsive, smooth gameplay
5. **Cost Optimization** - Identify waste and inefficiency

**Approach:**
- **Metrics** - Time-series numeric data (Prometheus)
- **Logs** - Structured event data (JSON logs)
- **Traces** - Distributed request flows (OpenTelemetry)
- **Profiles** - CPU/Memory profiling (on-demand)

---

## Table of Contents

1. [Metrics Strategy](#1-metrics-strategy)
2. [Dashboard Designs](#2-dashboard-designs)
3. [Alerting Rules](#3-alerting-rules)
4. [Logging Strategy](#4-logging-strategy)
5. [Tracing Strategy](#5-tracing-strategy)
6. [Implementation Roadmap](#6-implementation-roadmap)
7. [Tool Stack](#7-tool-stack)

---

## 1. Metrics Strategy

### 1.1 Metric Types

**Counters** - Monotonically increasing values
- Total requests, errors, messages sent
- Never decreases, only increases
- Rate of change most useful (req/sec)

**Gauges** - Point-in-time values
- Current CPU usage, memory, entity count
- Can go up or down
- Snapshot of current state

**Histograms** - Distribution of values
- Request latency, processing time
- Provides percentiles (p50, p95, p99)
- Most useful for SLO tracking

**Summaries** - Similar to histograms, pre-computed percentiles
- Use for high-cardinality data
- Less flexible, more efficient

### 1.2 Simulation Server Metrics

#### **Core Simulation Performance**

```rust
// Tick timing
simulation_tick_duration_seconds (Histogram)
  - Labels: none
  - Purpose: How long each simulation tick takes
  - SLO: p99 < 50ms (20 Hz requirement)
  - Alert: p99 > 45ms for 2 minutes

simulation_tick_rate_hz (Gauge)
  - Labels: none
  - Purpose: Actual tick rate achieved
  - SLO: > 19 Hz (allow 5% variance)
  - Alert: < 18 Hz for 1 minute

simulation_entities_total (Gauge)
  - Labels: type=[creature, plant, resource]
  - Purpose: Current entity count by type
  - SLO: None (informational)
  - Alert: None (capacity planning only)

simulation_entities_spawned_total (Counter)
  - Labels: type=[creature, plant, resource]
  - Purpose: Total entities created
  - SLO: None
  - Alert: Spike detection (>10x normal rate)

simulation_entities_despawned_total (Counter)
  - Labels: type=[creature, plant, resource], reason=[death, cleanup, other]
  - Purpose: Total entities removed
  - SLO: None
  - Alert: Mass extinction event (>80% despawned in 1 min)
```

#### **ECS System Performance**

```rust
simulation_system_duration_seconds (Histogram)
  - Labels: system=[physics, behavior, rendering_prep, spatial_grid, etc.]
  - Purpose: Per-system execution time
  - SLO: Sum of all systems p99 < 45ms
  - Alert: Any system p99 > 20ms

simulation_system_executions_total (Counter)
  - Labels: system=[...], status=[success, error]
  - Purpose: System execution count and error rate
  - SLO: Error rate < 0.1%
  - Alert: Error rate > 1% for any system
```

#### **Memory & Resources**

```rust
simulation_memory_bytes (Gauge)
  - Labels: type=[heap, stack, mmap]
  - Purpose: Memory usage breakdown
  - SLO: < 4 GB for 1M entities
  - Alert: > 6 GB

simulation_memory_allocations_total (Counter)
  - Labels: size_bucket=[small, medium, large]
  - Purpose: Track allocation patterns
  - SLO: None (performance debugging)
  - Alert: Spike > 2x normal rate

simulation_cpu_percent (Gauge)
  - Labels: core=[0, 1, 2, ...]
  - Purpose: Per-core CPU utilization
  - SLO: < 80% average
  - Alert: > 95% for 5 minutes
```

### 1.3 Streaming Worker Metrics

#### **Worker Thread Performance**

```rust
streaming_worker_processing_duration_seconds (Histogram)
  - Labels: stage=[total, spatial_filter, delta_encode, serialize, compress, publish]
  - Purpose: Breakdown of worker processing time
  - SLO: p99 < 30ms (30 Hz budget)
  - Alert: p99 > 28ms for 2 minutes

streaming_worker_lag_seconds (Gauge)
  - Labels: none
  - Purpose: How far behind worker is (channel buffer age)
  - SLO: < 0.1 seconds
  - Alert: > 0.5 seconds

streaming_worker_channel_buffer_size (Gauge)
  - Labels: none
  - Purpose: Pending snapshots in channel
  - SLO: < 3 frames
  - Alert: > 5 frames for 1 minute

streaming_worker_dropped_frames_total (Counter)
  - Labels: reason=[overload, error, nats_failure]
  - Purpose: Count of dropped streaming updates
  - SLO: < 0.1% of frames
  - Alert: > 1% dropped in 5 minutes
```

#### **Data Processing Metrics**

```rust
streaming_entities_total (Gauge)
  - Labels: stage=[input, after_spatial, after_delta, sent]
  - Purpose: Entity count at each filtering stage
  - SLO: None (informational)
  - Alert: None

streaming_entities_filtered_total (Counter)
  - Labels: reason=[out_of_region, unchanged]
  - Purpose: Why entities were filtered out
  - SLO: > 95% filtered (1M → 50k)
  - Alert: < 90% filtered (not optimizing enough)

streaming_delta_change_rate (Gauge)
  - Labels: region=[0.0, 0.1, 1.0, 1.1, ...]
  - Purpose: % entities changed per region
  - SLO: None (informational, expect ~30%)
  - Alert: None
```

#### **Serialization & Compression**

```rust
streaming_serialization_bytes (Histogram)
  - Labels: stage=[uncompressed, compressed], format=[flatbuffers]
  - Purpose: Size of serialized data
  - SLO: None (informational)
  - Alert: None

streaming_compression_ratio (Histogram)
  - Labels: algorithm=[lz4]
  - Purpose: Compression effectiveness
  - SLO: > 1.5:1 (expect ~2:1)
  - Alert: < 1.3:1 for 5 minutes (compression not working)

streaming_compression_duration_seconds (Histogram)
  - Labels: algorithm=[lz4]
  - Purpose: Time spent compressing
  - SLO: p99 < 2ms
  - Alert: p99 > 5ms
```

### 1.4 NATS Metrics

#### **Publisher (Simulation Side)**

```rust
nats_messages_published_total (Counter)
  - Labels: subject=[simulation.region.*, simulation.lifecycle.*, etc.]
  - Purpose: Message count by topic
  - SLO: ~30 msg/sec per region at 30 Hz
  - Alert: 0 messages for 10 seconds

nats_bytes_published_total (Counter)
  - Labels: subject=[...]
  - Purpose: Bandwidth by topic
  - SLO: < 10 MB/sec total
  - Alert: > 50 MB/sec (leak/bug)

nats_publish_duration_seconds (Histogram)
  - Labels: subject=[...]
  - Purpose: Time to publish message
  - SLO: p99 < 1ms
  - Alert: p99 > 5ms

nats_publish_errors_total (Counter)
  - Labels: error_type=[connection_lost, timeout, buffer_full]
  - Purpose: Publish failure tracking
  - SLO: < 0.01% error rate
  - Alert: Any error in 1 minute
```

#### **Subscriber (Broadcaster Side)**

```rust
nats_messages_received_total (Counter)
  - Labels: subject=[...]
  - Purpose: Message count received
  - SLO: Match published count (within 1%)
  - Alert: Mismatch > 5% for 2 minutes

nats_message_receive_latency_seconds (Histogram)
  - Labels: subject=[...]
  - Purpose: Network latency (publish to receive)
  - SLO: p99 < 5ms (local network)
  - Alert: p99 > 20ms

nats_subscription_lag_seconds (Gauge)
  - Labels: subject=[...]
  - Purpose: How far behind subscriber is
  - SLO: < 0.1 seconds
  - Alert: > 1 second
```

### 1.5 Broadcaster Metrics

#### **Broadcaster Core**

```rust
broadcaster_processing_duration_seconds (Histogram)
  - Labels: stage=[receive, decompress, decode, fanout]
  - Purpose: Processing time breakdown
  - SLO: p99 < 5ms
  - Alert: p99 > 10ms

broadcaster_clients_connected (Gauge)
  - Labels: none
  - Purpose: Number of portal clients connected
  - SLO: None (informational)
  - Alert: Sudden drop > 50% in 1 minute

broadcaster_messages_fanned_out_total (Counter)
  - Labels: region=[...]
  - Purpose: Messages sent to clients
  - SLO: None
  - Alert: 0 fanout with clients connected

broadcaster_bandwidth_bytes_per_second (Gauge)
  - Labels: direction=[inbound, outbound]
  - Purpose: Network throughput
  - SLO: Inbound ~4.5 MB/sec, Outbound varies by client count
  - Alert: Outbound > 100 MB/sec (check for leak)
```

#### **WebSocket Connections**

```rust
broadcaster_websocket_connections_total (Counter)
  - Labels: status=[opened, closed], reason=[normal, error, timeout]
  - Purpose: Connection lifecycle tracking
  - SLO: None
  - Alert: High error rate (>10% in 5 min)

broadcaster_websocket_message_duration_seconds (Histogram)
  - Labels: message_type=[entity_update, lifecycle, metadata]
  - Purpose: Time to send message to client
  - SLO: p99 < 10ms
  - Alert: p99 > 50ms

broadcaster_websocket_buffer_size_bytes (Gauge)
  - Labels: client_id=[...] (use cardinality limiter)
  - Purpose: Outbound buffer per client
  - SLO: < 1 MB per client
  - Alert: Any client > 5 MB (slow client, kick?)
```

### 1.6 Frontend (Portal) Metrics

**Note:** Client-side metrics reported back to backend

```javascript
portal_frame_duration_ms (Histogram)
  - Labels: none
  - Purpose: Client rendering frame time
  - SLO: p99 < 16ms (60 FPS)
  - Alert: p99 > 32ms (30 FPS) for 2 minutes

portal_entities_rendered (Gauge)
  - Labels: none
  - Purpose: Entities on screen
  - SLO: None (informational)
  - Alert: None

portal_websocket_latency_ms (Histogram)
  - Labels: none
  - Purpose: Round-trip time to broadcaster
  - SLO: p99 < 100ms
  - Alert: p99 > 200ms

portal_websocket_disconnects_total (Counter)
  - Labels: reason=[normal, error, timeout]
  - Purpose: Connection stability
  - SLO: < 1% disconnect rate
  - Alert: > 5% disconnect rate

portal_interpolation_error_units (Histogram)
  - Labels: none
  - Purpose: How far off predictions are
  - SLO: p95 < 5 world units
  - Alert: p95 > 20 units (prediction failing)
```

### 1.7 Database (PostgreSQL) Metrics

```sql
-- Standard PostgreSQL metrics
database_connections_active (Gauge)
database_query_duration_seconds (Histogram, label: query_type)
database_rows_returned_total (Counter)
database_transaction_duration_seconds (Histogram)
database_deadlocks_total (Counter)
database_cache_hit_ratio (Gauge) -- Should be > 0.95

-- Snapshot-specific
database_snapshot_write_duration_seconds (Histogram)
database_snapshot_size_bytes (Gauge)
database_snapshots_total (Counter, label: status=[success, error])
```

---

## 2. Dashboard Designs

### 2.1 Dashboard: Simulation Health (Primary)

**Purpose:** Real-time simulation performance overview

**Panels:**

```
┌─────────────────────────────────────────────────────┐
│ SIMULATION HEALTH                                   │
├─────────────────────────────────────────────────────┤
│                                                     │
│ [Tick Rate]         [Entity Count]      [Memory]   │
│  20.1 Hz             1,234,567         3.2 GB      │
│  ▲ Healthy           ▲ Normal          ▲ Normal    │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Tick Duration (p99) - Last 5 minutes               │
│ ┌─────────────────────────────────────┐            │
│ │        [Graph: Line chart]          │            │
│ │ 50ms ┤                               │            │
│ │      ├────────────────────           │            │
│ │ 25ms ┤                ▓▓▓▓▓          │            │
│ │      ├────────────────────           │            │
│ │   0ms└─────────────────────          │            │
│ │       0min      2min      5min       │            │
│ └─────────────────────────────────────┘            │
│ Current: 14.2ms  | Target: < 50ms                  │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ ECS System Performance (p99)                       │
│ ┌───────────────────────────────────┐              │
│ │ Physics           ████░░░░  8.2ms  │             │
│ │ Behavior          ██░░░░░░  3.1ms  │             │
│ │ Spatial Grid      █░░░░░░░  1.8ms  │             │
│ │ Rendering Prep    █░░░░░░░  1.2ms  │             │
│ │ Other             █░░░░░░░  0.9ms  │             │
│ └───────────────────────────────────┘              │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Entity Lifecycle - Last Hour                       │
│ ┌─────────────────────────────────────┐            │
│ │ Spawned:   +12,456  ▲ 2.3%          │            │
│ │ Despawned: -11,203  ▼ Normal        │            │
│ │ Net:       +1,253   ▲ Growing       │            │
│ └─────────────────────────────────────┘            │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Prometheus Queries:**

```promql
# Tick Rate
rate(simulation_tick_count_total[1m])

# Tick Duration p99
histogram_quantile(0.99, rate(simulation_tick_duration_seconds_bucket[5m]))

# Entity Count
simulation_entities_total

# Memory Usage
simulation_memory_bytes{type="heap"}

# ECS System Performance
histogram_quantile(0.99, rate(simulation_system_duration_seconds_bucket[5m])) by (system)

# Entity Lifecycle
rate(simulation_entities_spawned_total[1h])
rate(simulation_entities_despawned_total[1h])
```

---

### 2.2 Dashboard: Streaming Pipeline

**Purpose:** Monitor streaming architecture performance

**Panels:**

```
┌─────────────────────────────────────────────────────┐
│ STREAMING PIPELINE                                  │
├─────────────────────────────────────────────────────┤
│                                                     │
│ [Latency]          [Bandwidth]       [Drop Rate]   │
│  14.2ms             4.8 MB/sec        0.00%        │
│  ▲ Excellent        ▲ Normal          ▲ Perfect    │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Worker Processing Time - Last 5 minutes            │
│ ┌─────────────────────────────────────┐            │
│ │ [Stacked area chart]                │            │
│ │ 30ms ┤                               │            │
│ │      │ ┌────────────────────────┐   │            │
│ │ 20ms │ │ Publish                │   │            │
│ │      │ │ Compress               │   │            │
│ │ 10ms │ │ Serialize              │   │            │
│ │      │ │ Delta Encode           │   │            │
│ │   0ms└─┴─Spatial Filter─────────┘   │            │
│ └─────────────────────────────────────┘            │
│ Total: 9.8ms | Budget: 30ms                        │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Data Reduction Funnel                              │
│ ┌─────────────────────────────────────┐            │
│ │ Input:          1,000,000 entities  │            │
│ │      ▼ Spatial Filter (95%)         │            │
│ │ After Spatial:     50,000 entities  │            │
│ │      ▼ Delta Encode (70%)           │            │
│ │ After Delta:       15,000 entities  │            │
│ │      ▼ Compress (50%)               │            │
│ │ Final:          300 KB/frame        │            │
│ └─────────────────────────────────────┘            │
│ Reduction: 99.83% ✓                                │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ NATS Throughput by Region                          │
│ ┌───────────────────────────────────┐              │
│ │ region.0.0   ████████░  1.2 MB/s  │             │
│ │ region.0.1   ██████░░░  0.9 MB/s  │             │
│ │ region.1.0   █████████  1.4 MB/s  │             │
│ │ region.1.1   ███████░░  1.1 MB/s  │             │
│ └───────────────────────────────────┘              │
│ Total: 4.6 MB/s                                    │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Worker Channel Health                              │
│ ┌───────────────────────────────────┐              │
│ │ Buffer Size:    2 frames           │             │
│ │ Lag:            0.06 seconds       │             │
│ │ Dropped:        0 (last hour)      │             │
│ └───────────────────────────────────┘              │
│ Status: ✓ Healthy                                  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Prometheus Queries:**

```promql
# Latency (simulation to NATS)
histogram_quantile(0.99, rate(streaming_worker_processing_duration_seconds_bucket{stage="total"}[5m]))

# Bandwidth
sum(rate(nats_bytes_published_total[1m]))

# Drop Rate
rate(streaming_worker_dropped_frames_total[5m]) / rate(simulation_tick_count_total[5m])

# Worker Breakdown
rate(streaming_worker_processing_duration_seconds_sum{stage=~"spatial_filter|delta_encode|serialize|compress|publish"}[5m])
by (stage)

# Data Reduction
streaming_entities_total{stage="input"}
streaming_entities_total{stage="after_spatial"}
streaming_entities_total{stage="after_delta"}

# NATS by Region
rate(nats_bytes_published_total[1m]) by (subject)

# Worker Health
streaming_worker_channel_buffer_size
streaming_worker_lag_seconds
rate(streaming_worker_dropped_frames_total[1h])
```

---

### 2.3 Dashboard: End-to-End Latency

**Purpose:** Track latency from simulation to client browser

**Panels:**

```
┌─────────────────────────────────────────────────────┐
│ END-TO-END LATENCY                                  │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Total Latency (p95): 38ms                          │
│ Target: < 50ms (Animal reaction time) ✓            │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Latency Waterfall (p95)                            │
│ ┌─────────────────────────────────────┐            │
│ │ Simulation Tick       ████  14ms    │            │
│ │ Worker Processing     ███   10ms    │            │
│ │ NATS Transfer         █     2ms     │            │
│ │ Broadcaster Process   █     3ms     │            │
│ │ WebSocket Send        █     1ms     │            │
│ │ Internet Latency      ██    7ms     │            │
│ │ Client Render         █     1ms     │            │
│ └─────────────────────────────────────┘            │
│ Total: 38ms                                        │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Latency Distribution - Last Hour                   │
│ ┌─────────────────────────────────────┐            │
│ │      [Heatmap: latency over time]   │            │
│ │ 100ms┤░░░░░░░░░░░░░░░░░░░░░░░░░░░   │            │
│ │  75ms┤░░▓░░░░░░░░░░░░░░░░░░░░░░░░   │            │
│ │  50ms┤▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░   │            │
│ │  25ms┤████████████████████████████   │            │
│ │    0ms└────────────────────────────   │            │
│ │       0min  15min  30min  45min  60min│            │
│ └─────────────────────────────────────┘            │
│ p50: 28ms | p95: 38ms | p99: 45ms                  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Prometheus Queries:**

```promql
# Total Latency (requires tracing or computed metric)
histogram_quantile(0.95,
  rate(simulation_tick_duration_seconds_bucket[5m]) +
  rate(streaming_worker_processing_duration_seconds_bucket{stage="total"}[5m]) +
  rate(nats_message_receive_latency_seconds_bucket[5m]) +
  rate(broadcaster_processing_duration_seconds_bucket{stage="total"}[5m]) +
  rate(portal_websocket_latency_ms_bucket[5m]) / 1000
)

# Individual Component Latency
histogram_quantile(0.95, rate(simulation_tick_duration_seconds_bucket[5m]))
histogram_quantile(0.95, rate(streaming_worker_processing_duration_seconds_bucket[5m]))
# ... etc for each component
```

---

### 2.4 Dashboard: Broadcaster & Clients

**Purpose:** Monitor broadcaster service and client connections

```
┌─────────────────────────────────────────────────────┐
│ BROADCASTER & CLIENTS                               │
├─────────────────────────────────────────────────────┤
│                                                     │
│ [Connected]        [Bandwidth Out]    [Messages/s] │
│  1,247 clients      142 MB/sec        37,410       │
│  ▲ Normal           ▲ Normal          ▲ Normal     │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Client Connection Rate - Last Hour                 │
│ ┌─────────────────────────────────────┐            │
│ │ [Line chart: connects vs disconnects]│           │
│ │ Connects:    █████████████████       │            │
│ │ Disconnects: ██████████████          │            │
│ └─────────────────────────────────────┘            │
│ Net Growth: +124 clients                           │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Client Latency Distribution                        │
│ ┌───────────────────────────────────┐              │
│ │ < 50ms     ████████████  67%      │             │
│ │ 50-100ms   ████          23%      │             │
│ │ 100-200ms  ██            8%       │             │
│ │ > 200ms    █             2%       │             │
│ └───────────────────────────────────┘              │
│ p95: 87ms | Target: < 100ms ✓                     │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Slow Clients (Buffer > 1MB)                        │
│ ┌───────────────────────────────────┐              │
│ │ Currently: 3 clients               │             │
│ │ Action: Monitor, may kick if > 5MB│             │
│ └───────────────────────────────────┘              │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

### 2.5 Dashboard: Cost & Capacity

**Purpose:** Resource utilization and capacity planning

```
┌─────────────────────────────────────────────────────┐
│ COST & CAPACITY PLANNING                            │
├─────────────────────────────────────────────────────┤
│                                                     │
│ [CPU Usage]        [Memory Usage]    [Network]     │
│  42%                3.2 / 8.0 GB      4.8 MB/s     │
│  ▲ Normal           ▲ Normal          ▲ Normal     │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Entities per Dollar (Efficiency)                   │
│ ┌─────────────────────────────────────┐            │
│ │ Current:  250,000 entities/$        │            │
│ │ Target:   200,000 entities/$        │            │
│ │ Status:   ✓ 25% better than target  │            │
│ └─────────────────────────────────────┘            │
│                                                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│ Capacity Projection                                │
│ ┌─────────────────────────────────────┐            │
│ │ Current:  1.2M entities             │            │
│ │ Max (80%):  1.9M entities           │            │
│ │ Headroom:   700K entities           │            │
│ │ ETA to 80%: ~45 days (linear)       │            │
│ └─────────────────────────────────────┘            │
│ Action: Scale up before 30 days                    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

## 3. Alerting Rules

### 3.1 Critical Alerts (Page On-Call)

**Alert: SimulationDown**
```yaml
alert: SimulationDown
expr: up{job="simulation"} == 0
for: 1m
severity: critical
description: "Simulation server is down"
action: "Immediate investigation required"
```

**Alert: TickRateDegraded**
```yaml
alert: TickRateDegraded
expr: rate(simulation_tick_count_total[1m]) < 18
for: 2m
severity: critical
description: "Simulation tick rate below 18 Hz (target: 20 Hz)"
action: "Check CPU, memory, or blocking operations"
```

**Alert: HighTickLatency**
```yaml
alert: HighTickLatency
expr: histogram_quantile(0.99, rate(simulation_tick_duration_seconds_bucket[5m])) > 0.045
for: 2m
severity: critical
description: "Simulation tick p99 latency > 45ms (budget: 50ms)"
action: "Profile simulation, check for expensive systems"
```

**Alert: StreamingWorkerStalled**
```yaml
alert: StreamingWorkerStalled
expr: streaming_worker_channel_buffer_size > 5
for: 1m
severity: critical
description: "Streaming worker falling behind (buffer > 5 frames)"
action: "Worker overloaded, check processing time or reduce load"
```

**Alert: NATSPublishFailures**
```yaml
alert: NATSPublishFailures
expr: rate(nats_publish_errors_total[1m]) > 0
for: 30s
severity: critical
description: "NATS publish errors detected"
action: "Check NATS server health, network connectivity"
```

### 3.2 Warning Alerts (Slack/Email)

**Alert: HighMemoryUsage**
```yaml
alert: HighMemoryUsage
expr: simulation_memory_bytes{type="heap"} > 6e9  # 6 GB
for: 5m
severity: warning
description: "Simulation memory usage > 6 GB"
action: "Check for memory leaks, plan capacity increase"
```

**Alert: EntityPopulationSpike**
```yaml
alert: EntityPopulationSpike
expr: rate(simulation_entities_spawned_total[1m]) > 10 * rate(simulation_entities_spawned_total[1m] offset 10m)
for: 2m
severity: warning
description: "Entity spawn rate 10x normal"
action: "Possible bug, check spawner logic"
```

**Alert: CompressionIneffective**
```yaml
alert: CompressionIneffective
expr: streaming_compression_ratio < 1.3
for: 5m
severity: warning
description: "LZ4 compression ratio < 1.3:1 (expect 2:1)"
action: "Check data patterns, verify LZ4 working correctly"
```

**Alert: DataReductionLow**
```yaml
alert: DataReductionLow
expr: (streaming_entities_total{stage="sent"} / streaming_entities_total{stage="input"}) > 0.10
for: 5m
severity: warning
description: "Data reduction < 90% (expect 95-99%)"
action: "Check spatial filtering and delta encoding"
```

**Alert: ClientLatencyHigh**
```yaml
alert: ClientLatencyHigh
expr: histogram_quantile(0.95, rate(portal_websocket_latency_ms_bucket[5m])) > 100
for: 5m
severity: warning
description: "Client latency p95 > 100ms"
action: "Check network, broadcaster performance, or client distribution"
```

**Alert: HighDisconnectRate**
```yaml
alert: HighDisconnectRate
expr: rate(broadcaster_websocket_connections_total{status="closed",reason="error"}[5m]) / rate(broadcaster_websocket_connections_total{status="opened"}[5m]) > 0.05
for: 5m
severity: warning
description: "WebSocket error disconnect rate > 5%"
action: "Check broadcaster stability, network issues"
```

### 3.3 Info Alerts (Slack Only)

**Alert: ApproachingCapacity**
```yaml
alert: ApproachingCapacity
expr: simulation_entities_total > 1.5e6  # 1.5M
for: 10m
severity: info
description: "Entity count > 1.5M (capacity planning threshold)"
action: "Plan for scaling up infrastructure"
```

**Alert: NewRecordEntityCount**
```yaml
alert: NewRecordEntityCount
expr: simulation_entities_total > max_over_time(simulation_entities_total[7d])
for: 5m
severity: info
description: "New record entity count achieved!"
action: "Celebrate and monitor performance"
```

---

## 4. Logging Strategy

### 4.1 Log Levels

**ERROR** - Requires immediate attention
- System failures, panics, unrecoverable errors
- Examples: NATS connection lost, worker thread crashed

**WARN** - Potential issues, degraded performance
- Recovered errors, rate limiting, high latency
- Examples: Dropped frame, slow processing, high memory

**INFO** - Important events, lifecycle
- System startup/shutdown, milestones, major state changes
- Examples: Simulation started, snapshot saved, 1M entities reached

**DEBUG** - Detailed diagnostic information
- Only in development or troubleshooting
- Examples: Entity spawned, system executed, message sent

**TRACE** - Very verbose, every operation
- Rarely used, extreme debugging only
- Examples: Every position update, every message

### 4.2 Structured Logging Format

**Use JSON for structured logs:**

```rust
// Rust (using tracing crate)
use tracing::{info, warn, error};

info!(
    target: "simulation::tick",
    tick = tick_num,
    duration_ms = duration.as_millis(),
    entities = entity_count,
    "Simulation tick completed"
);

warn!(
    target: "streaming::worker",
    buffer_size = buffer.len(),
    lag_ms = lag.as_millis(),
    "Streaming worker falling behind"
);

error!(
    target: "nats::publisher",
    error = %err,
    subject = %subject,
    retry_count = retries,
    "Failed to publish NATS message"
);
```

**Output format:**
```json
{
  "timestamp": "2025-11-04T15:32:14.123Z",
  "level": "INFO",
  "target": "simulation::tick",
  "fields": {
    "tick": 123456,
    "duration_ms": 14,
    "entities": 1234567
  },
  "message": "Simulation tick completed"
}
```

### 4.3 Key Log Events

**Simulation:**
```rust
// Startup
info!("Simulation starting", version = VERSION, config = ?config);

// Tick Performance
info!("Tick completed", tick = num, duration_ms = dur, entities = count);

// Entity Lifecycle
debug!("Entity spawned", entity_id = id, position = ?pos, type = "creature");
debug!("Entity despawned", entity_id = id, reason = "death", age_seconds = age);

// Critical Events
warn!("High tick latency", duration_ms = dur, target_ms = 50, entities = count);
error!("System panic", system = "physics", error = %err);
```

**Streaming Worker:**
```rust
// Worker Lifecycle
info!("Streaming worker started", nats_url = %url, regions = region_count);

// Processing
debug!("Frame processed",
    entities_in = input,
    entities_filtered = filtered,
    entities_sent = sent,
    duration_ms = dur
);

// Performance
warn!("Slow compression", duration_ms = dur, uncompressed_kb = unc, ratio = ratio);

// Errors
error!("NATS publish failed", subject = %subj, error = %err, retry = retry_num);
```

**Broadcaster:**
```typescript
// Client Connections
logger.info('Client connected', {
  clientId,
  ip: req.ip,
  userAgent: req.headers['user-agent']
});

logger.info('Client disconnected', {
  clientId,
  duration_seconds: duration,
  reason
});

// Message Processing
logger.debug('Region update received', {
  region,
  entities: count,
  compressed_bytes: size
});

// Errors
logger.error('Decompression failed', {
  region,
  error: err.message,
  compressed_bytes: size
});
```

### 4.4 Log Retention

- **Simulation:** 7 days (high volume)
- **Broadcaster:** 14 days (moderate volume)
- **Frontend:** 30 days (low volume, user-facing)
- **ERROR level:** 90 days (all services)

### 4.5 Log Aggregation

**Tool:** Grafana Loki or ELK Stack

**Queries:**
```logql
# All errors in last hour
{job=~"simulation|broadcaster"} |= "level=ERROR" |> last 1h

# Slow ticks
{job="simulation"} |= "tick completed" | json | duration_ms > 40

# NATS failures
{target="nats::publisher"} |= "failed"

# Client disconnects
{job="broadcaster"} |= "disconnected" | json | reason != "normal"
```

---

## 5. Tracing Strategy

### 5.1 Distributed Tracing

**Tool:** OpenTelemetry + Jaeger

**Purpose:** Track request flow across services

**Trace Example:**
```
Simulation Tick #123456 [15ms total]
├─ Update Physics [8ms]
├─ Update Behavior [3ms]
├─ Update Spatial Grid [2ms]
└─ Stream to NATS [2ms]
   └─ Streaming Worker Processing [10ms, parallel]
      ├─ Spatial Filter [2ms]
      ├─ Delta Encode [3ms]
      ├─ Serialize [3ms]
      ├─ Compress [1ms]
      └─ NATS Publish [1ms]
         └─ NATS Network [2ms]
            └─ Broadcaster Receive [3ms]
               ├─ Decompress [0.5ms]
               ├─ Decode FlatBuffers [0.5ms]
               └─ WebSocket Fanout [2ms]
                  └─ Client Receive [varies]
```

### 5.2 Span Instrumentation

**Simulation (Rust):**
```rust
use tracing::{span, Level};

#[tracing::instrument(level = "info")]
fn update_simulation(world: &mut World, dt: f32) {
    let tick_span = span!(Level::INFO, "simulation_tick", tick = world.tick);
    let _enter = tick_span.enter();

    // Nested spans
    {
        let _physics = span!(Level::DEBUG, "physics_update").entered();
        update_physics(world, dt);
    }

    {
        let _behavior = span!(Level::DEBUG, "behavior_update").entered();
        update_behavior(world);
    }

    // ...
}
```

**Broadcaster (TypeScript):**
```typescript
import { trace } from '@opentelemetry/api';

const tracer = trace.getTracer('broadcaster');

async function processMessage(msg: NatsMessage) {
  const span = tracer.startSpan('process_nats_message', {
    attributes: { subject: msg.subject }
  });

  try {
    const decompressed = await decompress(msg.data);
    span.addEvent('decompressed', { size: decompressed.length });

    const entities = decode(decompressed);
    span.addEvent('decoded', { entity_count: entities.length });

    await fanout(entities);
    span.addEvent('fanned_out');

    span.setStatus({ code: SpanStatusCode.OK });
  } catch (err) {
    span.recordException(err);
    span.setStatus({ code: SpanStatusCode.ERROR });
  } finally {
    span.end();
  }
}
```

### 5.3 Sampling Strategy

**Production:** 1% sampling (high volume)
```yaml
sampling:
  default: 0.01  # 1%
  rules:
    - service: simulation
      sample_rate: 0.01
    - service: broadcaster
      sample_rate: 0.05  # 5% (less volume)
```

**Errors:** Always sample (100%)
```yaml
sampling:
  always_sample_on_error: true
```

**High Latency:** Always sample (debug)
```yaml
sampling:
  rules:
    - attribute: latency_ms
      threshold: 100
      sample_rate: 1.0  # 100%
```

---

## 6. Implementation Roadmap

### Phase 1: Foundation (Sprint 6)

**Week 1: Simulation Metrics**
- [ ] Add `prometheus` crate to simulation
- [ ] Implement core metrics:
  - `simulation_tick_duration_seconds`
  - `simulation_entities_total`
  - `simulation_memory_bytes`
- [ ] Expose metrics endpoint (`:9090/metrics`)
- [ ] Create "Simulation Health" dashboard

**Week 2: Streaming Metrics**
- [ ] Add metrics to `StreamingWorker`:
  - `streaming_worker_processing_duration_seconds`
  - `streaming_worker_channel_buffer_size`
  - `streaming_entities_total`
- [ ] Add NATS metrics:
  - `nats_messages_published_total`
  - `nats_publish_errors_total`
- [ ] Create "Streaming Pipeline" dashboard

### Phase 2: Alerting (Sprint 7)

**Week 3: Critical Alerts**
- [ ] Set up Alertmanager
- [ ] Configure PagerDuty integration
- [ ] Implement critical alerts:
  - `SimulationDown`
  - `TickRateDegraded`
  - `StreamingWorkerStalled`
- [ ] Test alert delivery

**Week 4: Warning Alerts**
- [ ] Configure Slack integration
- [ ] Implement warning alerts:
  - `HighMemoryUsage`
  - `CompressionIneffective`
  - `ClientLatencyHigh`
- [ ] Document runbooks

### Phase 3: Logging (Sprint 8)

**Week 5: Structured Logging**
- [ ] Replace `println!` with `tracing` crate
- [ ] Add JSON formatter
- [ ] Implement key log events:
  - Startup/shutdown
  - Tick completion
  - Errors and warnings
- [ ] Set up log aggregation (Loki)

**Week 6: Log Analysis**
- [ ] Create log dashboards in Grafana
- [ ] Set up log-based alerts
- [ ] Document common log queries

### Phase 4: Tracing (Sprint 9-10)

**Week 7: OpenTelemetry Integration**
- [ ] Add `opentelemetry` crates to Rust
- [ ] Add `@opentelemetry` packages to TypeScript
- [ ] Instrument critical paths:
  - Simulation tick
  - Streaming worker
  - NATS publish/subscribe

**Week 8: Trace Analysis**
- [ ] Set up Jaeger backend
- [ ] Configure sampling (1% default)
- [ ] Create trace-based dashboards
- [ ] Document trace queries

---

## 7. Tool Stack

### 7.1 Metrics

**Prometheus** - Time-series database
- Industry standard
- Powerful query language (PromQL)
- Excellent Rust/Node.js clients
- Self-hosted or managed (Grafana Cloud)

**Grafana** - Visualization and dashboards
- Best-in-class dashboards
- Alerting built-in
- Integrates with everything
- Free and open source

**Rust Crate:** `prometheus = "0.13"`
```rust
use prometheus::{Histogram, Counter, Gauge, Registry};

lazy_static! {
    static ref TICK_DURATION: Histogram = Histogram::new(
        "simulation_tick_duration_seconds",
        "Simulation tick duration"
    ).unwrap();
}

// Usage
let timer = TICK_DURATION.start_timer();
simulation.update(dt);
timer.observe_duration();
```

**Node.js Package:** `prom-client`
```typescript
import { Histogram, Counter } from 'prom-client';

const processingDuration = new Histogram({
  name: 'broadcaster_processing_duration_seconds',
  help: 'Broadcaster processing time',
  labelNames: ['stage']
});

// Usage
const end = processingDuration.startTimer({ stage: 'decompress' });
await decompress(data);
end();
```

### 7.2 Logging

**Loki** - Log aggregation (Grafana Labs)
- Like Prometheus, but for logs
- Integrates seamlessly with Grafana
- Cost-effective (indexes labels, not content)
- Easy to operate

**Alternative:** ELK Stack (Elasticsearch + Logstash + Kibana)
- More powerful search
- Higher operational overhead
- Better for complex log analysis

**Rust Crate:** `tracing = "0.1"` + `tracing-subscriber`
```rust
use tracing::{info, warn, error};

// Setup
tracing_subscriber::fmt()
    .json()
    .with_target(true)
    .with_current_span(true)
    .init();

// Usage
info!(tick = 123, duration_ms = 14, "Tick completed");
```

**Node.js Package:** `winston` or `pino`
```typescript
import pino from 'pino';

const logger = pino({
  level: 'info',
  formatters: { level: (label) => ({ level: label }) }
});

// Usage
logger.info({ clientId, region }, 'Client connected');
```

### 7.3 Tracing

**OpenTelemetry** - Vendor-neutral standard
- Works with Jaeger, Zipkin, Datadog, etc.
- Auto-instrumentation available
- Future-proof choice

**Jaeger** - Trace backend
- Open source
- Good UI for trace analysis
- Easy to get started

**Rust Crate:** `opentelemetry = "0.21"` + `opentelemetry-jaeger`
```rust
use opentelemetry::trace::{Tracer, SpanKind};
use tracing_opentelemetry::OpenTelemetryLayer;

// Setup
let tracer = opentelemetry_jaeger::new_agent_pipeline()
    .with_service_name("simulation")
    .install_simple()?;

// Tracing works automatically with tracing crate
```

**Node.js Package:** `@opentelemetry/sdk-node`
```typescript
import { NodeSDK } from '@opentelemetry/sdk-node';
import { JaegerExporter } from '@opentelemetry/exporter-jaeger';

const sdk = new NodeSDK({
  serviceName: 'broadcaster',
  traceExporter: new JaegerExporter()
});

sdk.start();
```

### 7.4 Alerting

**Alertmanager** - Prometheus alerting
- Routes alerts to appropriate channels
- Grouping, throttling, silencing
- PagerDuty, Slack, email integrations

**PagerDuty** - On-call management (critical alerts)
**Slack** - Team notifications (warnings)
**Email** - Low-priority info

---

## 8. Future Metrics (Phase 2+)

### 8.1 Game Balance Metrics

```
player_actions_total (Counter)
  - Labels: action_type=[spawn, interact, modify]
  - Purpose: Track player engagement

economy_balance_units (Gauge)
  - Labels: resource_type=[biomass, energy, ...]
  - Purpose: Monitor economic balance

gameplay_events_total (Counter)
  - Labels: event_type=[extinction, population_boom, ...]
  - Purpose: Track emergent events
```

### 8.2 Scientific Metrics

```
species_diversity_index (Gauge)
  - Purpose: Shannon diversity index
  - Track ecosystem health

genetic_drift_rate (Gauge)
  - Purpose: Rate of genetic change
  - Monitor evolution speed

trophic_level_distribution (Histogram)
  - Purpose: Food chain analysis
  - Ensure ecological balance
```

### 8.3 Business Metrics

```
daily_active_users (Gauge)
monthly_active_users (Gauge)
session_duration_seconds (Histogram)
user_retention_rate (Gauge)
revenue_per_user (Gauge, if monetized)
```

---

## Summary

This instrumentation plan provides:

1. **Comprehensive Metrics** - 50+ metrics covering all system components
2. **Actionable Dashboards** - 5 dashboards for different audiences
3. **Intelligent Alerting** - 15+ alerts with clear severity levels
4. **Structured Logging** - JSON logs with proper levels and context
5. **Distributed Tracing** - End-to-end visibility across services
6. **Phased Implementation** - Realistic 8-week rollout plan
7. **Modern Tool Stack** - Prometheus, Grafana, Loki, OpenTelemetry

**Next Steps:**
1. Review and approve this plan with team
2. Begin Phase 1 (Foundation) in Sprint 6
3. Iterate based on operational learnings
4. Expand to Phase 2 metrics as system matures

**Questions? Contact:** Backend Simulation Team, DevOps Daria

---

**Document Version:** 1.0
**Last Updated:** 2025-11-04
**Next Review:** After Phase 1 implementation (Sprint 7)
