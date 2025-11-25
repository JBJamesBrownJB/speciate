import { describe, it, expect } from "vitest";
import { Mesh, Shader, Geometry, Buffer, BufferUsage } from "pixi.js";

describe("Layer 0: Minimal Mesh Creation (Unit Tests)", () => {
  describe("Test A: aQuadVertex Attribute Approach", () => {
    it("creates mesh with aQuadVertex buffer without errors", () => {
      const geometry = new Geometry();

      const quadBuffer = new Buffer({
        data: new Float32Array([
          0, 0,
          1, 0,
          0, 1,
          1, 1,
        ]),
        usage: BufferUsage.VERTEX,
      });

      geometry.addAttribute("aPosition", {
        buffer: quadBuffer,
        format: "float32x2",
        stride: 8,
        offset: 0,
      });

      const vertexSrc = `#version 300 es
        precision highp float;

        in vec2 aPosition;

        uniform mat3 projectionMatrix;
        uniform mat3 translationMatrix;

        void main() {
          vec2 screenPos = (aPosition - 0.5) * 200.0;
          gl_Position = vec4((projectionMatrix * translationMatrix * vec3(screenPos, 1.0)).xy, 0.0, 1.0);
        }
      `;

      const fragmentSrc = `#version 300 es
        precision highp float;

        out vec4 fragColor;

        void main() {
          fragColor = vec4(1.0, 0.0, 0.0, 1.0);
        }
      `;

      const shader = Shader.from({
        gl: {
          vertex: vertexSrc,
          fragment: fragmentSrc,
        },
        resources: {},
      });

      const mesh = new Mesh({ geometry, shader });

      expect(mesh).toBeDefined();
      expect(mesh.geometry).toBe(geometry);
      expect(mesh.shader).toBe(shader);
      expect(geometry.attributes.aPosition).toBeDefined();
    });
  });

  describe("Test B: gl_VertexID Computation Approach", () => {
    it("creates mesh with gl_VertexID computation without errors", () => {
      const geometry = new Geometry();

      const dummyBuffer = new Buffer({
        data: new Float32Array(4),
        usage: BufferUsage.VERTEX,
      });

      geometry.addAttribute("aDummy", {
        buffer: dummyBuffer,
        format: "float32",
        stride: 4,
        offset: 0,
      });

      const vertexSrc = `#version 300 es
        precision highp float;

        in float aDummy;

        uniform mat3 projectionMatrix;
        uniform mat3 translationMatrix;

        void main() {
          int vertexId = gl_VertexID % 4;
          vec2 localPos = vec2(0.0);

          if (vertexId == 0) {
            localPos = vec2(-0.5, -0.5);
          } else if (vertexId == 1) {
            localPos = vec2(0.5, -0.5);
          } else if (vertexId == 2) {
            localPos = vec2(-0.5, 0.5);
          } else {
            localPos = vec2(0.5, 0.5);
          }

          vec2 screenPos = localPos * 200.0;
          gl_Position = vec4((projectionMatrix * translationMatrix * vec3(screenPos, 1.0)).xy, 0.0, 1.0);
        }
      `;

      const fragmentSrc = `#version 300 es
        precision highp float;

        out vec4 fragColor;

        void main() {
          fragColor = vec4(0.0, 0.0, 1.0, 1.0);
        }
      `;

      const shader = Shader.from({
        gl: {
          vertex: vertexSrc,
          fragment: fragmentSrc,
        },
        resources: {},
      });

      const mesh = new Mesh({ geometry, shader });

      expect(mesh).toBeDefined();
      expect(mesh.geometry).toBe(geometry);
      expect(mesh.shader).toBe(shader);
      expect(geometry.attributes.aDummy).toBeDefined();
    });
  });
});
