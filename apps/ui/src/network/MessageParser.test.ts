import { describe, it, expect, vi } from 'vitest';
import { MessageParser } from './MessageParser';

describe('MessageParser', () => {
  const parser = new MessageParser();

  describe('parse', () => {
    it('should parse WorldState message', () => {
      const rawMessage = JSON.stringify({
        type: 'WorldState',
        entities: [
          {
            id: 'entity1',
            position: { x: 100, y: 100 },
            orientation: 0,
            radius: 10,
          },
        ],
      });

      const result = parser.parse(rawMessage);

      expect(result.type).toBe('worldState');
      if (result.type === 'worldState') {
        expect(result.data.entities).toHaveLength(1);
        expect(result.data.entities[0].id).toBe('entity1');
      }
    });

    it('should parse EntityUpdate message', () => {
      const rawMessage = JSON.stringify({
        type: 'EntityUpdate',
        entity_id: 'entity1',
        position: { x: 100, y: 100 },
        orientation: 0,
      });

      const result = parser.parse(rawMessage);

      expect(result.type).toBe('entityUpdate');
      if (result.type === 'entityUpdate') {
        expect(result.data.entity_id).toBe('entity1');
      }
    });

    it('should return unknown for invalid JSON', () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

      const rawMessage = 'invalid json';
      const result = parser.parse(rawMessage);

      expect(result.type).toBe('unknown');
      expect(consoleSpy).toHaveBeenCalled();

      consoleSpy.mockRestore();
    });

    it('should return unknown for unrecognized message type', () => {
      const rawMessage = JSON.stringify({
        type: 'UnknownType',
        data: {},
      });

      const result = parser.parse(rawMessage);

      expect(result.type).toBe('unknown');
    });
  });
});
