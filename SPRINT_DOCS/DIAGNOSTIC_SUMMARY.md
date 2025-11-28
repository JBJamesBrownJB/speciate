# Creature Rendering Diagnostic Summary - Sprint 14

**Date:** 2025-11-25
**Branch:** `feat/sprint-14-interpolation-perception`
**Status:** 🟡 RENDERING PIPELINE WORKS - Coordinate Transform Issue Remains

---

## 🎉 MAJOR BREAKTHROUGH

**We proved the rendering pipeline works!** A hardcoded red square rendered successfully, confirming:
- ✅ PixiJS v8 Mesh/Geometry/Shader setup is correct
- ✅ Instanced rendering works
- ✅ GLSL ES 3.0 shaders compile and execute
- ✅ No z-index issues - mesh is visible

---

## ✅ Issues Fixed

### 1. Shader/Geometry Attribute Mismatch ⚠️ **CRITICAL FIX**
**Problem:** Shader declared `aQuadVertex` but used `gl_VertexID` to compute vertices
**Root Cause:** Incompatible approaches - can't declare an attribute and not use it
**Fix:** Changed shader to actually use `aQuadVertex`:
```glsl
vec2 localPos = (aQuadVertex - 0.5) * aSize;
vTextureCoord = aQuadVertex;
```
**File:** `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts:175-181`

### 2. Unused `aCreatureId` Attribute
**Problem:** Declared in shader and geometry but never used, wasting 4 bytes per creature
**Fix:** Removed from both shader (line 150) and geometry (lines 122-128)
**Impact:** Stride reduced from 32 → 28 bytes (saves 100KB for 25K creatures)
**File:** `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts`

### 3. Missing Geometry Configuration
**Problem:** PixiJS didn't know how to render the geometry
**Fixes Applied:**
- Set `geometry.topology = 'triangle-strip'` (line 124)
- Set `geometry.vertexCount = 4` (line 129)
- Set `geometry.instanceCount` dynamically (line 291)

### 4. Buffer Format Alignment
**Problem:** InterpolationBufferManager had 8 floats per creature (including unused ID)
**Fix:** Updated to 7 floats per creature:
```
[startX, startY, endX, endY, startRot, endRot, size]
```
**File:** `apps/portal/src/rendering/InterpolationBufferManager.ts:6-14`

---

## 🔴 Remaining Issue: Coordinate Transform

### The Problem
**Red square renders → Creatures with real coordinates don't**

This proves the issue is **coordinate space transformation**, not the rendering pipeline.

### Why It Fails
Creatures are positioned in **world coordinates** (meters, -50m to +50m), but the shader needs to transform them correctly to screen space.

**Current approach (doesn't work):**
```glsl
vec3 screenPos = projectionMatrix * translationMatrix * vec3(finalPosWorld, 1.0);
gl_Position = vec4(screenPos.xy, 0.0, 1.0);
```

**Problem:** PixiJS v8's `projectionMatrix` and `translationMatrix` uniforms **may not be auto-bound** for custom shaders like they were in v7.

### The Solution: Manual Camera Uniforms

Pass camera transform explicitly to the shader:

**1. Add camera uniforms to shader:**
```glsl
uniform vec2 uCameraPos;      // Camera position (meters)
uniform float uCameraZoom;     // Pixels per meter
uniform vec2 uViewportSize;    // Screen size (pixels)
```

**2. Update uniforms every frame:**
```typescript
render(deltaMs: number, camera: Camera, viewport: Viewport): void {
  this.shader.resources.uCameraPos = { value: [camera.x, camera.y], type: 'vec2<f32>' };
  this.shader.resources.uCameraZoom = { value: camera.zoom, type: 'f32' };
  this.shader.resources.uViewportSize = { value: [viewport.width, viewport.height], type: 'vec2<f32>' };

  // ... rest of render logic
}
```

**3. Transform in shader:**
```glsl
void main() {
  // World position (meters)
  vec2 worldPos = mix(aStartPos, aEndPos, uInterpolation);

  // ... rotation logic ...

  // Camera transform: world meters → screen pixels
  vec2 viewPos = (worldPos - uCameraPos) * uCameraZoom;
  vec2 screenPos = viewPos + uViewportSize * 0.5; // Center at screen center

  // Screen pixels → NDC (-1 to +1)
  vec2 ndc = (screenPos / uViewportSize) * 2.0 - 1.0;
  ndc.y *= -1.0; // Flip Y (PixiJS uses top-left origin)

  gl_Position = vec4(ndc, 0.0, 1.0);
}
```

---

## 📊 Current State

**Rendering Pipeline:**
- ✅ Geometry: 4 vertices (quad), triangle-strip topology
- ✅ Instancing: instanceCount set dynamically per frame
- ✅ Shader: GLSL ES 3.0, proper attribute usage
- ✅ Buffer: 28-byte stride, 7 floats per creature
- ✅ Data Flow: NAPI → IPC → Renderer (confirmed working)

**What Works:**
- Hardcoded red square renders at screen center
- No PixiJS warnings or errors
- 2,501 creatures loading from save state

**What Doesn't Work:**
- Real creature positions (coordinate transform issue)

---

## 🔧 Next Steps

### Option A: Manual Camera Uniforms (Recommended)
1. Add camera uniforms to shader resources
2. Update uniforms in `render()` method
3. Implement camera transform in vertex shader
4. Test with live creatures

**Estimated Time:** 15-20 minutes
**Risk:** Low (proven approach)

### Option B: Debug PixiJS Built-in Matrices
1. Log `projectionMatrix` and `translationMatrix` values
2. Verify they're being set correctly
3. Check PixiJS v8 docs for custom shader matrix binding

**Estimated Time:** 30-45 minutes
**Risk:** Medium (may be PixiJS internal issue)

---

## 📁 Modified Files

**Core Implementation:**
- `apps/portal/src/rendering/InterpolatedCreatureRenderer.ts` - Shader/geometry fixes
- `apps/portal/src/rendering/InterpolationBufferManager.ts` - Buffer format update

**Tests:**
- `apps/portal/src/rendering/MinimalMeshTest.test.ts` - Created Layer 0 isolation tests
- `apps/portal/src/rendering/InterpolatedCreatureRenderer.test.ts` - Fixed mock texture

**Documentation:**
- `/home/dev/.claude/plans/goofy-jumping-stream.md` - Diagnostic plan

---

## 🧪 Debug Commands

**Restart dev server:**
```bash
cd apps/portal
lsof -ti:5173 | xargs kill -9 2>/dev/null || true
npm run dev
```

**Run tests:**
```bash
npm test -- InterpolatedCreatureRenderer
npm test -- MinimalMeshTest
```

**Type check:**
```bash
npm run type-check
```

---

## 💡 Key Learnings

1. **Isolation testing works!** Starting with the simplest possible case (hardcoded red quad) immediately identified the coordinate transform as the issue.

2. **PixiJS v8 is stricter** about attribute usage - declared attributes must be used in the shader.

3. **Always verify the full pipeline first** before debugging data flow - we wasted time checking buffer updates when the issue was coordinate math.

4. **Manual camera control is cleaner** - gives full visibility into transformations instead of relying on PixiJS magic.

---

## 🎯 Success Criteria

When coordinate transforms are fixed, we should see:
- ✅ 2,501+ creatures rendering smoothly
- ✅ Creatures positioned correctly in world space
- ✅ Camera pan/zoom working correctly
- ✅ 60 FPS interpolation between 22.2Hz simulation ticks
- ✅ No PixiJS warnings or errors

---

**Next session:** Implement Option A (manual camera uniforms) to complete the rendering system.
