# Stress-Induced Tunnel Vision

## Concept

Acute stress narrows perception (both FOV and range), simulating biological tunnel vision.

## Biological Basis

- Sympathetic nervous system activation
- Cortisol and adrenaline prioritize immediate threats
- Peripheral processing suppressed
- Attention locks onto escape route or threat source

## Proposed Implementation

```
stress_factor = current_stress / max_stress
fov_modifier = 1.0 - (0.4 * stress_factor)   // Up to 40% FOV reduction
range_modifier = 1.0 - (0.3 * stress_factor) // Up to 30% range reduction

effective_fov = base_fov * fov_modifier
effective_range = base_range * range_modifier
```

## Biological Bounds

- Maximum FOV reduction: 40% (tunnel vision is real but not blindness)
- Maximum range reduction: 30% (hypervigilance partially compensates)
- Recovery time: 5-10 seconds after stressor removal (cortisol decay)

## Gameplay Implications

- Stressed creatures vulnerable to flanking attacks
- Creates "panic" moments during predator encounters
- Recovery time prevents instant re-engagement

## Integration

Would require:
- Stress component on creatures
- Stress accumulation from threats, injuries, starvation
- Stress decay over time

## Source

Zoologist-tom consultation, 2025-11-30
