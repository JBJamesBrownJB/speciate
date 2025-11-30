import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { SelectionHighlight } from './SelectionHighlight';
import { Container } from 'pixi.js';

describe('SelectionHighlight', () => {
  let highlight: SelectionHighlight;
  let container: Container;

  beforeEach(() => {
    container = new Container();
    highlight = new SelectionHighlight(container);
  });

  afterEach(() => {
    highlight.destroy();
  });

  describe('initial state', () => {
    it('should not be visible initially', () => {
      expect(highlight.isVisible()).toBe(false);
    });
  });

  describe('show', () => {
    it('should set visible state to true', () => {
      highlight.show(100, 200, 15);
      expect(highlight.isVisible()).toBe(true);
    });

    it('should add graphics to container', () => {
      highlight.show(100, 200, 15);
      expect(container.children.length).toBe(1);
    });
  });

  describe('hide', () => {
    it('should set visible state to false', () => {
      highlight.show(100, 200, 15);
      highlight.hide();
      expect(highlight.isVisible()).toBe(false);
    });
  });

  describe('updatePosition', () => {
    it('should not throw when visible', () => {
      highlight.show(100, 200, 15);
      expect(() => highlight.updatePosition(150, 250)).not.toThrow();
    });

    it('should not throw when hidden', () => {
      expect(() => highlight.updatePosition(150, 250)).not.toThrow();
    });
  });

  describe('update (animation)', () => {
    it('should not throw when visible', () => {
      highlight.show(100, 200, 15);
      expect(() => highlight.update(16)).not.toThrow();
    });

    it('should not throw when hidden', () => {
      expect(() => highlight.update(16)).not.toThrow();
    });
  });

  describe('destroy', () => {
    it('should not throw', () => {
      highlight.show(100, 200, 15);
      expect(() => highlight.destroy()).not.toThrow();
    });
  });
});
