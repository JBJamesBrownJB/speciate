import { vi } from 'vitest';
import type { Texture, Particle } from 'pixi.js';

export function createMockCanvas(): {
  canvas: HTMLCanvasElement;
  context: CanvasRenderingContext2D;
  spies: {
    clearRect: ReturnType<typeof vi.fn>;
    beginPath: ReturnType<typeof vi.fn>;
    moveTo: ReturnType<typeof vi.fn>;
    lineTo: ReturnType<typeof vi.fn>;
    stroke: ReturnType<typeof vi.fn>;
    setLineDash: ReturnType<typeof vi.fn>;
    scale: ReturnType<typeof vi.fn>;
  };
} {
  const spies = {
    clearRect: vi.fn(),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    stroke: vi.fn(),
    setLineDash: vi.fn(),
    scale: vi.fn(),
  };

  const contextProperties = {
    strokeStyle: '',
    lineWidth: 0,
  };

  const context = {
    ...spies,
    get strokeStyle() { return contextProperties.strokeStyle; },
    set strokeStyle(value: string | CanvasGradient | CanvasPattern) {
      contextProperties.strokeStyle = typeof value === 'string' ? value : '';
    },
    get lineWidth() { return contextProperties.lineWidth; },
    set lineWidth(value: number) { contextProperties.lineWidth = value; },
  } as unknown as CanvasRenderingContext2D;

  const canvas = {
    getContext: vi.fn().mockReturnValue(context),
    getBoundingClientRect: vi.fn().mockReturnValue({
      width: 200,
      height: 60,
    }),
    width: 0,
    height: 0,
    style: {},
  } as unknown as HTMLCanvasElement;

  return { canvas, context, spies };
}

export function createMockElement(initialText = ''): HTMLElement {
  return {
    textContent: initialText,
    style: {},
  } as HTMLElement;
}

export function createMockTexture(width = 32, height = 32): Texture {
  return {
    width,
    height,
  } as Texture;
}

export function createMockParticle(): Particle {
  return {
    x: 0,
    y: 0,
    rotation: 0,
    scaleX: 1,
    scaleY: 1,
  } as Particle;
}

export function mockDocumentGetElementById(
  elementMap: Record<string, HTMLElement | null>
): void {
  vi.spyOn(document, 'getElementById').mockImplementation((id: string) => {
    return elementMap[id] ?? null;
  });
}

export function setDevicePixelRatio(ratio: number): void {
  Object.defineProperty(window, 'devicePixelRatio', {
    writable: true,
    configurable: true,
    value: ratio,
  });
}

export function resetDevicePixelRatio(): void {
  delete (window as any).devicePixelRatio;
}
