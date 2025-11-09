# Backlight as Feature Indicator

## Overview

The Portal's ambient backlight glow (the glowing border around the viewport) is a **functional UI element**, not just a decorative effect. It serves as a **persistent, ambient state indicator** that provides immediate visual feedback about the application's current mode or connection status without requiring the player to look at text or icons.

This design pattern should be considered whenever implementing new features, modes, or states in the Portal.

## Current Implementation

### Technical Details

The backlight is implemented using CSS `box-shadow` on the `#canvas-container` element. It consists of:

1. **Inner shadows**: Dark vignette effect (always present)
2. **Outer glow**: Color-coded ambient light that changes based on state
3. **Pulse animations**: Subtle pulsing for error/warning states to draw attention

**Performance Optimization**: Animations use layered `box-shadow` in keyframes (GPU-accelerated) rather than pseudo-elements or direct DOM manipulation (CPU-intensive). Multiple shadow layers create depth and pulse effects.

### Current States (3-State System)

| State | Color | Animation | Purpose | CSS Class |
|-------|-------|-----------|---------|-----------|
| **Connecting/Reconnecting** | Orange | Pulse | Any connection attempt or reconnection | `.glow-connecting-reconnecting` |
| **Connected** | Green | Subtle wave | Normal operational state | `.glow-connected` |
| **Error/Disconnected** | Red | Pulse | Manual disconnect or critical error (rare) | `.glow-error` |

**Color Rationale**:
- Follows "traffic light" UX pattern (Orange = caution/warning, Green = normal/healthy, Red = error/stop)
- **Cyan is reserved for future features** (e.g., High Altitude Drone mode, special modes)
- Orange used for both Connecting and Reconnecting since they represent the same user experience (waiting for connection)
- Combined with HUD text status for accessibility

**Animation Details**:
- **Orange pulse**: 2-second cycle, noticeable intensity change to draw attention
- **Green wave**: 4-second cycle, very subtle shimmer effect with directional glow shift (0px → 20px → 25px)
- **Red pulse**: 2-second cycle, strong intensity change for critical states

## Design Principle: Ambient Awareness

The backlight provides **ambient awareness** - information the player can perceive without direct attention:

- **Peripheral vision**: Color changes are noticeable without looking directly at them
- **Non-intrusive**: Doesn't block gameplay or require acknowledgment
- **Subtle animations**: Pulse effects draw attention to problems without being annoying
- **Glanceable**: Even when focused on the simulation, players can sense the mode

This makes it ideal for:
- **Connection status**: Already implemented
- **Mode indicators**: See "High Altitude Drone" example below
- **Time states**: Fast-forward, pause, recording
- **Alert levels**: Warnings, errors, confirmations
- **Gameplay modes**: Day/night, seasons, special events

## Feature Integration Guidelines

### When to Use the Backlight

**✅ DO use the backlight for:**
- **Persistent modes** that last more than a few seconds
- **Critical states** that affect how the player should interpret what they see
- **Mode transitions** that change camera behavior or data display
- **Background processes** that the player should be aware of (recording, syncing, etc.)
- **States that benefit from ambient awareness** (player doesn't need to actively check)

**❌ DON'T use the backlight for:**
- **Transient notifications** (toasts, one-time messages)
- **Minor UI states** (hover effects, focus states)
- **Very frequent changes** (would be distracting)
- **Information that requires precise reading** (use HUD or overlays instead)

### How to Add a New Backlight State

**For static colors (no animation):**

1. **Define the color** in `index.html` CSS:
   ```css
   #canvas-container.glow-yourmode {
     /* Override default box-shadow with your color */
     box-shadow: inset 0 0 60px rgba(0,0,0,0.9),
                 inset 0 0 120px rgba(0,0,0,0.7),
                 0 0 40px rgba(R,G,B,ALPHA);
   }
   ```

**For animated colors (pulse, wave, etc.):**

1. **Define the animation** in `index.html` CSS:
   ```css
   #canvas-container.glow-yourmode {
     animation: yourmode-animation 3s ease-in-out infinite;
   }

   @keyframes yourmode-animation {
     0%, 100% {
       box-shadow: inset 0 0 60px rgba(0,0,0,0.9),
                   inset 0 0 120px rgba(0,0,0,0.7),
                   0 0 40px rgba(R,G,B,ALPHA1),
                   0 0 80px rgba(R,G,B,ALPHA2);  /* Optional: extra layer for depth */
     }
     50% {
       box-shadow: inset 0 0 60px rgba(0,0,0,0.9),
                   inset 0 0 120px rgba(0,0,0,0.7),
                   0 0 50px rgba(R,G,B,ALPHA3),   /* Increase blur/opacity */
                   0 0 100px rgba(R,G,B,ALPHA4);
     }
   }
   ```

2. **Apply the class** in your TypeScript code:
   ```typescript
   const container = document.getElementById("canvas-container");
   if (container) {
     // Remove all existing glow classes
     container.classList.remove(
       "glow-connecting-reconnecting",
       "glow-connected",
       "glow-error"
     );
     // Add your new class
     container.classList.add("glow-yourmode");
   }
   ```

3. **Document it** in this file under "Current States" table

**Tips**:
- Use 2-4 layers of box-shadow for depth (close blur + distant blur)
- Keep animation cycles between 2-5 seconds
- Subtle changes work best (alpha differences of 0.05-0.15)
- Test visibility on different monitors/brightness settings

### Color Selection Guidelines

- **Cool colors** (blue, cyan, teal): Normal, calm, strategic modes
- **Warm colors** (yellow, orange): Warning, caution, transition states
- **Red**: Errors, disconnection, critical alerts
- **Green**: Confirmation, success, healthy states
- **Purple/Magenta**: Special modes, premium features
- **White/Gray**: Neutral, paused, inactive states

**Avoid**:
- Colors too similar to existing states
- Oversaturated colors (keep alpha around 0.1-0.2 for subtlety)
- Flickering or rapid color changes (jarring)

## Example: High Altitude Drone Mode

From `HighAltDrone.md` (line 17):

> "When Strategic View is active, the glowing border of the viewport (the UI vignette) will change to a distinct color (e.g., a "digital" blue) to provide clear visual feedback that the player is in 'Drone Mode'."

**Proposed Implementation** (using reserved cyan color):

```css
#canvas-container.glow-drone {
  box-shadow: inset 0 0 60px rgba(0,0,0,0.9),
              inset 0 0 120px rgba(0,0,0,0.7),
              0 0 40px rgba(0,255,255,0.15);  /* Cyan - reserved for drone mode */
}
```

**User Experience**:
- Player scrolls out with mouse wheel
- Camera transitions past normal zoom limit
- Backlight smoothly transitions from green → cyan
- Player immediately understands they're in a different mode
- Entities change to symbolic representation (circles)
- Scrolling back in transitions backlight back to green

**Benefits**:
- **No modal dialogs** needed to explain mode change
- **Consistent with theme**: Cyan feels "high-tech" and distinct from normal green
- **Reinforces spatial metaphor**: "Going higher" = different view = different color
- **Reversible feedback**: Color returns to normal when exiting mode
- **Reserved color**: Cyan is specifically saved for this premium feature

## Future State Ideas

Consider backlight integration for these potential features:

| Feature | Suggested Color | Animation | Rationale |
|---------|----------------|-----------|-----------|
| **High Altitude Drone** | **Cyan `rgba(0,255,255,0.15)` (RESERVED)** | None or subtle shift | Tech upgrade, strategic view, distinct from normal |
| **Pause Mode** | Gray `rgba(128,128,128,0.1)` | None | Desaturated = inactive |
| **Fast-Forward** | Purple `rgba(200,100,255,0.15)` | Rapid pulse | Accelerated time = energetic color |
| **Recording** | Red `rgba(255,50,50,0.1)` | Slow pulse | Like a recording indicator |
| **Replay Mode** | Amber `rgba(255,191,0,0.12)` | None | Reviewing past = warm historical tone |
| **God Mode** | Gold `rgba(255,215,0,0.2)` | Gentle shimmer | Premium/powerful = gold |
| **Night Cycle** | Deep Blue `rgba(20,20,80,0.15)` | None | Matches in-game time |
| **Day Cycle** | Light Yellow `rgba(255,250,200,0.08)` | None | Matches in-game time |
| **Tutorial Mode** | Light Blue `rgba(100,200,255,0.12)` | None | Learning = calm blue (avoid green confusion) |
| **PvP Arena** | Red-Orange `rgba(255,100,50,0.15)` | Pulse | Competitive = intense |
| **Creative Mode** | Rainbow | Hue-rotate | Freedom = full spectrum |

## Accessibility Considerations

**Color Alone is Not Enough**: The backlight should **reinforce** other indicators, not replace them:

- ✅ Combine with HUD text (as currently implemented for connection status)
- ✅ Use icons for mode indicators
- ✅ Provide text labels in settings/tooltips
- ✅ Consider users with color blindness (red/green issues most common)
- ✅ Ensure sufficient contrast and visibility

**Why this matters**:
- ~8% of males, ~0.5% of females have some form of color blindness
- Red/green distinction is most problematic
- Our current implementation combines backlight color with HUD text status ✓

## Architecture Notes

### Performance

- CSS transitions are GPU-accelerated ✓
- Pseudo-element animations avoid expensive box-shadow repaints ✓
- Class-based switching is highly performant ✓
- Minimal JavaScript overhead (just class manipulation) ✓

### Maintainability

- Visual design (colors) separated in CSS, not hardcoded in TypeScript
- Self-documenting class names (`glow-connected`, `glow-drone`)
- Follows existing pattern used for HUD status indicator
- Easy to add new states: 1 CSS class + 1 code switch case

### Extensibility

- No limit to number of states
- Can combine with other visual effects (borders, overlays, etc.)
- Could be exposed to user preferences in future (color customization, intensity)
- Could sync with external devices (RGB keyboard/mouse via Web USB in future)

## Conclusion

The backlight is a **subtle but powerful** UI element that enhances player experience through ambient awareness. When designing new features, always ask:

> **"Does this feature have a persistent mode or state that would benefit from ambient visual feedback?"**

If yes, consider integrating it with the backlight system.

---

**Related Files**:
- `/workspace/apps/portal/index.html` - CSS definitions
- `/workspace/apps/portal/src/main.ts` - State handling logic
- `/workspace/docs/gameplay_ideas/HighAltDrone.md` - First feature to reference backlight integration
