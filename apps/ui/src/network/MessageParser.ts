import type {
  ServerMessage,
  WorldStateMessage,
  EntityUpdateMessage,
} from '../types/messages';

export type ParsedMessage =
  | { type: 'worldState'; data: WorldStateMessage }
  | { type: 'entityUpdate'; data: EntityUpdateMessage }
  | { type: 'unknown'; data: unknown };

export class MessageParser {
  parse(rawMessage: string): ParsedMessage {
    try {
      const message = JSON.parse(rawMessage) as ServerMessage;

      if (this.isWorldState(message)) {
        return { type: 'worldState', data: message };
      }

      if (this.isEntityUpdate(message)) {
        return { type: 'entityUpdate', data: message };
      }

      return { type: 'unknown', data: message };
    } catch (error) {
      console.error('Failed to parse message:', error);
      return { type: 'unknown', data: rawMessage };
    }
  }

  private isWorldState(message: ServerMessage): message is WorldStateMessage {
    return message.type === 'WorldState' && Array.isArray(message.entities);
  }

  private isEntityUpdate(message: ServerMessage): message is EntityUpdateMessage {
    return (
      message.type === 'EntityUpdate' &&
      typeof message.entity_id === 'string' &&
      typeof message.position === 'object'
    );
  }
}
