import { describe, it, expect } from 'vitest';
import { ErrorCodes } from '../protocol/messages';
import type { ClientMessage, ServerMessage } from '../protocol/messages';

describe('Protocol Messages', () => {
  describe('ErrorCodes', () => {
    it('should have correct UNAUTHORIZED code', () => {
      expect(ErrorCodes.UNAUTHORIZED).toBe(401);
    });

    it('should have correct FORBIDDEN code', () => {
      expect(ErrorCodes.FORBIDDEN).toBe(403);
    });

    it('should have correct NOT_FOUND code', () => {
      expect(ErrorCodes.NOT_FOUND).toBe(404);
    });

    it('should have correct INTERNAL_ERROR code', () => {
      expect(ErrorCodes.INTERNAL_ERROR).toBe(500);
    });

    it('should have correct INVALID_MESSAGE code', () => {
      expect(ErrorCodes.INVALID_MESSAGE).toBe(400);
    });
  });

  describe('ClientMessage types', () => {
    it('should allow Auth message type', () => {
      const msg: ClientMessage = { type: 'Auth', token: 'test' };
      expect(msg.type).toBe('Auth');
    });

    it('should allow Subscribe message type', () => {
      const msg: ClientMessage = { type: 'Subscribe', subject: 'test', id: 1 };
      expect(msg.type).toBe('Subscribe');
    });

    it('should allow Unsubscribe message type', () => {
      const msg: ClientMessage = { type: 'Unsubscribe', id: 1 };
      expect(msg.type).toBe('Unsubscribe');
    });

    it('should allow Publish message type', () => {
      const msg: ClientMessage = { type: 'Publish', subject: 'test', payload: new Uint8Array() };
      expect(msg.type).toBe('Publish');
    });

    it('should allow Request message type', () => {
      const msg: ClientMessage = { 
        type: 'Request', 
        subject: 'test', 
        payload: new Uint8Array(),
        timeoutMs: 5000,
        requestId: 1
      };
      expect(msg.type).toBe('Request');
    });

    it('should allow Ping message type', () => {
      const msg: ClientMessage = { type: 'Ping' };
      expect(msg.type).toBe('Ping');
    });
  });

  describe('ServerMessage types', () => {
    it('should allow AuthOk message type', () => {
      const msg: ServerMessage = { type: 'AuthOk', sessionId: 'session-123' };
      expect(msg.type).toBe('AuthOk');
    });

    it('should allow AuthError message type', () => {
      const msg: ServerMessage = { type: 'AuthError', reason: 'Invalid token' };
      expect(msg.type).toBe('AuthError');
    });

    it('should allow SubscribeOk message type', () => {
      const msg: ServerMessage = { type: 'SubscribeOk', id: 1 };
      expect(msg.type).toBe('SubscribeOk');
    });

    it('should allow SubscribeError message type', () => {
      const msg: ServerMessage = { type: 'SubscribeError', id: 1, reason: 'Denied' };
      expect(msg.type).toBe('SubscribeError');
    });

    it('should allow Message type', () => {
      const msg: ServerMessage = { 
        type: 'Message', 
        subscriptionId: 1,
        subject: 'test',
        payload: new Uint8Array()
      };
      expect(msg.type).toBe('Message');
    });

    it('should allow Response type', () => {
      const msg: ServerMessage = { 
        type: 'Response', 
        requestId: 1,
        payload: new Uint8Array()
      };
      expect(msg.type).toBe('Response');
    });

    it('should allow RequestError type', () => {
      const msg: ServerMessage = { type: 'RequestError', requestId: 1, reason: 'Timeout' };
      expect(msg.type).toBe('RequestError');
    });

    it('should allow Error type', () => {
      const msg: ServerMessage = { type: 'Error', code: 500, message: 'Internal error' };
      expect(msg.type).toBe('Error');
    });

    it('should allow Pong type', () => {
      const msg: ServerMessage = { type: 'Pong' };
      expect(msg.type).toBe('Pong');
    });
  });
});
