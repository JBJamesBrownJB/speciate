import {
  Geometry,
  Buffer,
  BufferUsage,
  Shader,
  Mesh,
  UniformGroup,
  type Texture,
} from "pixi.js";
import { interpDiag } from "./InterpolationDiagnostics";
import { SnapshotInterpolator } from "./SnapshotInterpolator";
import { writeInterleavedSegment } from "./interleavedBuffer";
import type { CreatureData } from "@/types/GameState";
import { getTickIntervalMs } from "@/core/constants";

/** A copied, stable snapshot of one frame's creatures (the renderer must not hold
 *  references to ElectronIPCClient's reused objects). */
type Snapshot = Array<{ id: number; x: number; y: number; rotation: number; size: number }>;

export class InterpolatedCreatureRenderer {
  private static readonly FLOATS_PER_CREATURE = 7;
  private static readonly DEFAULT_MAX_CREATURES = 200_000;

  private geometry: Geometry;
  private shader: Shader;
  private mesh: Mesh;

  // Double buffering for GPU stall prevention (pre-allocated to max capacity)
  private vertexBuffers: [Float32Array, Float32Array];
  private vertexBufferCapacity: number;
  private currentBufferIndex: number = 0;

  // Render-in-the-past playout: buffer snapshots, interpolate ~1 tick behind so the
  // tween always has a target to roll into (no end-of-tween stall, no reset-on-arrival).
  private interpolator = new SnapshotInterpolator<Snapshot>({ tickIntervalMs: 50 });
  private latestSnapshot: Snapshot | null = null; // newest data — count/visibility/warm-up
  private uploadedTo: Snapshot | null = null; // which `to` snapshot is currently on the GPU
  private dirty: boolean = false;

  constructor(texture: Texture, maxCreatures: number = InterpolatedCreatureRenderer.DEFAULT_MAX_CREATURES) {
    // Pre-allocate double buffers to max capacity (avoids GC pressure during spawning)
    this.vertexBufferCapacity = maxCreatures;
    const bufferSize = maxCreatures * InterpolatedCreatureRenderer.FLOATS_PER_CREATURE;
    this.vertexBuffers = [new Float32Array(bufferSize), new Float32Array(bufferSize)];

    // Create custom geometry
    this.geometry = this.createGeometry();

    // Create interpolation shader (Phase 2B: GPU interpolation)
    this.shader = this.createShader(texture);

    // Create mesh (v8 API) - suppress type error for custom Geometry
    this.mesh = new Mesh({ geometry: this.geometry, shader: this.shader }) as any;
    this.mesh.visible = false; // Hide until we have creatures
  }

  private createGeometry(): Geometry {
    const geometry = new Geometry();

    // Base quad geometry (4 vertices for triangle strip)
    // Vertices in 0-1 range, used by shader to compute quad corners
    const quadBuffer = new Buffer({
      data: new Float32Array([
        0, 0,  // vertex 0 (BL)
        1, 0,  // vertex 1 (BR)
        0, 1,  // vertex 2 (TL)
        1, 1,  // vertex 3 (TR)
      ]),
      usage: BufferUsage.VERTEX,
    });

    geometry.addAttribute('aQuadVertex', {
      buffer: quadBuffer,
      format: 'float32x2',
      stride: 8,
      offset: 0,
      instance: false,  // Per-vertex, not per-instance
    });

    // Create empty instance buffer upfront (prevents PixiJS render error before first update)
    const emptyBuffer = new Buffer({
      data: new Float32Array(0),
      usage: BufferUsage.VERTEX | BufferUsage.COPY_DST,
    });

    const STRIDE = 28; // 7 floats × 4 bytes (removed aCreatureId)

    // Add all 5 instance attributes
    geometry.addAttribute('aStartPos', {
      buffer: emptyBuffer,
      format: 'float32x2',
      stride: STRIDE,
      offset: 0,
      instance: true,
    });
    geometry.addAttribute('aEndPos', {
      buffer: emptyBuffer,
      format: 'float32x2',
      stride: STRIDE,
      offset: 8,
      instance: true,
    });
    geometry.addAttribute('aStartRot', {
      buffer: emptyBuffer,
      format: 'float32',
      stride: STRIDE,
      offset: 16,
      instance: true,
    });
    geometry.addAttribute('aEndRot', {
      buffer: emptyBuffer,
      format: 'float32',
      stride: STRIDE,
      offset: 20,
      instance: true,
    });
    geometry.addAttribute('aSize', {
      buffer: emptyBuffer,
      format: 'float32',
      stride: STRIDE,
      offset: 24,
      instance: true,
    });

    // Set topology for triangle strip rendering (quad = 4 vertices)
    geometry.topology = 'triangle-strip';

    // CRITICAL: Tell geometry how many vertices to draw per instance
    // Without this, PixiJS doesn't know to draw the 4 quad vertices
    // Type assertions needed for PixiJS v8 API
    (geometry as any).indexBuffer = null; // No index buffer, using vertex order directly
    (geometry as any).vertexCount = 4; // Draw 4 vertices per instance (the quad)

    geometry.instanceCount = 0;

    return geometry;
  }

  private createShader(texture: Texture): Shader {
    const vertexSrc = `#version 300 es
      precision highp float;

      in vec2 aQuadVertex;  // Base quad vertices (0-1 range, per-vertex)
      in vec2 aStartPos;    // Instance attributes below
      in vec2 aEndPos;
      in float aStartRot;
      in float aEndRot;
      in float aSize;

      uniform float uInterpolation;
      uniform vec2 uCameraPos;      // Camera position in world meters
      uniform float uCameraZoom;     // Pixels per meter
      uniform vec2 uViewportSize;    // Screen size in pixels
      uniform float uTextureAspectRatio;  // Height/width ratio of texture

      out vec2 vTextureCoord;

      // Shortest-path rotation interpolation (prevents 360° spins)
      float shortestPathRotation(float start, float end, float t) {
        float diff = end - start;
        const float PI = 3.14159265359;
        const float TWO_PI = 6.28318530718;

        // Wrap to [-PI, PI] range
        if (diff > PI) diff -= TWO_PI;
        if (diff < -PI) diff += TWO_PI;

        return start + diff * t;
      }

      void main() {
        // Interpolate world position (in meters)
        vec2 worldPos = mix(aStartPos, aEndPos, uInterpolation);
        float rotation = shortestPathRotation(aStartRot, aEndRot, uInterpolation);

        // Create quad vertex in local space with aspect-corrected dimensions
        vec2 quadSize = vec2(aSize, aSize * uTextureAspectRatio);
        vec2 localPos = (aQuadVertex - 0.5) * quadSize;
        vTextureCoord = aQuadVertex;

        // Apply rotation
        float cosRot = cos(rotation);
        float sinRot = sin(rotation);
        vec2 rotatedPos = vec2(
          localPos.x * cosRot - localPos.y * sinRot,
          localPos.x * sinRot + localPos.y * cosRot
        );

        // World position (meters)
        vec2 finalPosWorld = worldPos + rotatedPos;

        // Manual camera transform: world meters → screen pixels → NDC
        // 1. Translate to camera-relative position (meters)
        vec2 viewPos = finalPosWorld - uCameraPos;

        // 2. Scale by zoom (meters → pixels)
        vec2 screenPos = viewPos * uCameraZoom;

        // 3. Offset to screen center
        screenPos += uViewportSize * 0.5;

        // 4. Convert to NDC (normalized device coordinates: -1 to +1)
        vec2 ndc = (screenPos / uViewportSize) * 2.0 - 1.0;
        ndc.y *= -1.0; // Flip Y axis (PixiJS uses top-left origin, NDC uses bottom-left)

        gl_Position = vec4(ndc, 0.0, 1.0);
      }
    `;

    const fragmentSrc = `#version 300 es
      precision highp float;

      uniform sampler2D uTexture;
      in vec2 vTextureCoord;
      out vec4 fragColor;

      void main() {
        fragColor = texture(uTexture, vTextureCoord);
      }
    `;

    // PixiJS v8 Shader API - uniforms must be wrapped in UniformGroup with typed values
    const uniforms = new UniformGroup({
      uInterpolation: { value: 0.0, type: 'f32' },
      uCameraPos: { value: new Float32Array([0, 0]), type: 'vec2<f32>' },
      uCameraZoom: { value: 10.0, type: 'f32' },
      uViewportSize: { value: new Float32Array([800, 600]), type: 'vec2<f32>' },
      uTextureAspectRatio: { value: texture.height / texture.width, type: 'f32' },
    });

    return Shader.from({
      gl: {
        vertex: vertexSrc,
        fragment: fragmentSrc,
      },
      resources: {
        uTexture: texture.source,
        uniforms,
      },
    });
  }

  initialize(creatures: CreatureData[]): void {
    this.interpolator.reset();
    this.latestSnapshot = null;
    this.uploadedTo = null;
    this.pushSnapshot(creatures);
  }

  onSimulationTick(creatures: CreatureData[]): void {
    this.pushSnapshot(creatures);
  }

  /** Copy a frame into a stable snapshot and buffer it. Never resets the tween. */
  private pushSnapshot(creatures: CreatureData[]): void {
    const snap: Snapshot = new Array(creatures.length);
    for (let i = 0; i < creatures.length; i++) {
      const c = creatures[i];
      snap[i] = { id: c.id, x: c.x, y: c.y, rotation: c.rotation, size: c.size };
    }
    this.interpolator.push(snap);
    this.latestSnapshot = snap;
    this.dirty = true;
    this.mesh.visible = snap.length > 0;
  }

  render(
    deltaMS: number,
    cameraX: number,
    cameraY: number,
    cameraZoom: number,
    viewportWidth: number,
    viewportHeight: number
  ): void {
    this.interpolator.advance(deltaMS);

    // Show the interpolated pair once playback has started, else the latest snapshot
    // statically (warm-up before we have enough buffered to interpolate).
    const seg = this.interpolator.current();
    let from: Snapshot | null = null;
    let to: Snapshot | null = null;
    let alpha = 0;
    if (seg) {
      from = seg.from;
      to = seg.to;
      alpha = seg.alpha;
    } else if (this.latestSnapshot) {
      from = this.latestSnapshot;
      to = this.latestSnapshot;
    }

    // DEV-only: a frame clamped at 1.0 is a "frozen" frame (the stall we're removing).
    if (import.meta.env.DEV) interpDiag.recordFrame(alpha >= 1.0);

    // Rebuild + upload only when the displayed `to` changes (rollover or new snapshot).
    if (to && to !== this.uploadedTo) {
      this.buildAndUpload(from!, to);
      this.uploadedTo = to;
      this.dirty = false;
    }

    const uniforms = (this.shader.resources.uniforms as UniformGroup).uniforms;
    uniforms.uInterpolation = alpha;
    (uniforms.uCameraPos as Float32Array)[0] = cameraX;
    (uniforms.uCameraPos as Float32Array)[1] = cameraY;
    uniforms.uCameraZoom = cameraZoom;
    (uniforms.uViewportSize as Float32Array)[0] = viewportWidth;
    (uniforms.uViewportSize as Float32Array)[1] = viewportHeight;
  }

  /** Build the interleaved start/end buffer for the `from`->`to` segment and upload it. */
  private buildAndUpload(from: Snapshot, to: Snapshot): void {
    const creatureCount = to.length;

    // When cleared, just set instance count to 0 - no buffer update needed
    if (creatureCount === 0) {
      this.geometry.instanceCount = 0;
      return;
    }

    // Ensure capacity (only allocates if exceeding pre-allocated size)
    this.ensureVertexBufferCapacity(creatureCount);

    // Swap to inactive buffer (prevents GPU stall while GPU reads active buffer) and
    // write the id-matched start/end data directly into it (no intermediate copy).
    const nextBufferIndex = 1 - this.currentBufferIndex;
    writeInterleavedSegment(from, to, this.vertexBuffers[nextBufferIndex]);
    this.currentBufferIndex = nextBufferIndex;

    // Update the GPU buffer with a subarray view of just the used portion for PixiJS
    const usedLength = creatureCount * InterpolatedCreatureRenderer.FLOATS_PER_CREATURE;
    const activeBufferView = this.vertexBuffers[this.currentBufferIndex].subarray(0, usedLength);
    const pixiBuffer = this.geometry.getBuffer('aStartPos');
    if (pixiBuffer) {
      pixiBuffer.data = activeBufferView;
      pixiBuffer.update();
    }

    // Update instance count for instanced rendering
    this.geometry.instanceCount = creatureCount;
  }

  private ensureVertexBufferCapacity(requiredCount: number): void {
    if (requiredCount <= this.vertexBufferCapacity) {
      return;
    }

    const newCapacity = Math.max(requiredCount, this.vertexBufferCapacity * 2);
    const newSize = newCapacity * InterpolatedCreatureRenderer.FLOATS_PER_CREATURE;

    const newBuffer0 = new Float32Array(newSize);
    const newBuffer1 = new Float32Array(newSize);

    newBuffer0.set(this.vertexBuffers[0]);
    newBuffer1.set(this.vertexBuffers[1]);

    this.vertexBuffers = [newBuffer0, newBuffer1];
    this.vertexBufferCapacity = newCapacity;
  }

  getMesh(): Mesh {
    return this.mesh;
  }

  getCreatureCount(): number {
    return this.latestSnapshot?.length ?? 0;
  }

  getUniforms(): Record<string, unknown> {
    return (this.shader.resources.uniforms as UniformGroup).uniforms as Record<string, unknown>;
  }

  /** True when the newest snapshot has not yet been uploaded to the GPU. */
  isBufferDirty(): boolean {
    return this.dirty;
  }

  setTickRate(tickRateHz: number): void {
    this.interpolator.setTickInterval(getTickIntervalMs(tickRateHz));
  }
}
