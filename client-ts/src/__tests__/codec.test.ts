import { describe, it, expect } from 'vitest';
import { encodeClientMessage, decodeServerMessage } from '../protocol/codec';
import type { ClientMessage } from '../protocol/messages';

describe('MessageCodec', () => {
  describe('encodeClient', () => {
    it('should encode Auth message', () => {
      const msg: ClientMessage = { type: 'Auth', token: 'test.jwt.token' };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
      expect(encoded.length).toBeGreaterThan(0);
      expect(encoded[0]).toBe(0); // Auth tag
    });

    it('should encode Subscribe message', () => {
      const msg: ClientMessage = { type: 'Subscribe', subject: 'test.subject', id: 42 };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
      expect(encoded[0]).toBe(1); // Subscribe tag
    });

    it('should encode Unsubscribe message', () => {
      const msg: ClientMessage = { type: 'Unsubscribe', id: 123 };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
      expect(encoded[0]).toBe(2); // Unsubscribe tag
    });

    it('should encode Publish message', () => {
      const msg: ClientMessage = { 
        type: 'Publish', 
        subject: 'events.test', 
        payload: new Uint8Array([1, 2, 3, 4, 5]) 
      };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
      expect(encoded[0]).toBe(3); // Publish tag
    });

    it('should encode Request message', () => {
      const msg: ClientMessage = { 
        type: 'Request', 
        subject: 'api.test', 
        payload: new Uint8Array([1, 2, 3]),
        timeoutMs: 5000,
        requestId: 999
      };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
      expect(encoded[0]).toBe(4); // Request tag
    });

    it('should encode Ping message', () => {
      const msg: ClientMessage = { type: 'Ping' };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
      expect(encoded.length).toBe(1);
      expect(encoded[0]).toBe(5); // Ping tag
    });

    it('should encode empty payload', () => {
      const msg: ClientMessage = { 
        type: 'Publish', 
        subject: 'test', 
        payload: new Uint8Array([]) 
      };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
    });

    it('should encode unicode subject', () => {
      const msg: ClientMessage = { 
        type: 'Subscribe', 
        subject: 'test.unicode', 
        id: 1 
      };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
    });

    it('should encode large payload', () => {
      const largePayload = new Uint8Array(10000).fill(42);
      const msg: ClientMessage = { 
        type: 'Publish', 
        subject: 'large.message', 
        payload: largePayload 
      };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded).toBeInstanceOf(Uint8Array);
      expect(encoded.length).toBeGreaterThan(10000);
    });
  });

  describe('decodeServer', () => {
    // Helper to create mock server messages for testing decoder
    const createAuthOkMessage = (sessionId: string): Uint8Array => {
      const encoder = new TextEncoder();
      const sessionIdBytes = encoder.encode(sessionId);
      const lenBytes = encodeVarInt(sessionIdBytes.length);
      
      const result = new Uint8Array(1 + lenBytes.length + sessionIdBytes.length);
      result[0] = 0; // AuthOk tag
      result.set(lenBytes, 1);
      result.set(sessionIdBytes, 1 + lenBytes.length);
      return result;
    };

    const createPongMessage = (): Uint8Array => {
      return new Uint8Array([8]); // Pong tag
    };

    const createErrorMessage = (code: number, message: string): Uint8Array => {
      const encoder = new TextEncoder();
      const messageBytes = encoder.encode(message);
      const messageLenBytes = encodeVarInt(messageBytes.length);
      
      const result = new Uint8Array(1 + 4 + messageLenBytes.length + messageBytes.length);
      let offset = 0;
      result[offset++] = 7; // Error tag
      
      // code as u32 little-endian
      const codeView = new DataView(result.buffer, result.byteOffset + offset, 4);
      codeView.setUint32(0, code, true);
      offset += 4;
      
      result.set(messageLenBytes, offset);
      offset += messageLenBytes.length;
      result.set(messageBytes, offset);
      
      return result;
    };

    it('should decode AuthOk message', () => {
      const data = createAuthOkMessage('session-123');
      const decoded = decodeServerMessage(data);
      
      expect(decoded.type).toBe('AuthOk');
      if (decoded.type === 'AuthOk') {
        expect(decoded.sessionId).toBe('session-123');
      }
    });

    it('should decode Pong message', () => {
      const data = createPongMessage();
      const decoded = decodeServerMessage(data);
      
      expect(decoded.type).toBe('Pong');
    });

    it('should decode Error message', () => {
      const data = createErrorMessage(500, 'Internal error');
      const decoded = decodeServerMessage(data);
      
      expect(decoded.type).toBe('Error');
      if (decoded.type === 'Error') {
        expect(decoded.code).toBe(500);
        expect(decoded.message).toBe('Internal error');
      }
    });

    it('should throw on unknown tag', () => {
      const data = new Uint8Array([255]);
      expect(() => decodeServerMessage(data)).toThrow('Unknown server message tag');
    });
  });

  describe('varint encoding', () => {
    it('should encode small numbers correctly', () => {
      const msg: ClientMessage = { type: 'Subscribe', subject: 'a', id: 1 };
      const encoded = encodeClientMessage(msg);
      
      // Tag (1) + length of 'a' (1 varint) + 'a' (1) + id (8 bytes)
      expect(encoded.length).toBe(1 + 1 + 1 + 8);
    });

    it('should encode larger varints correctly', () => {
      const longSubject = 'a'.repeat(200);
      const msg: ClientMessage = { type: 'Subscribe', subject: longSubject, id: 1 };
      const encoded = encodeClientMessage(msg);
      
      // 200 needs 2 bytes in varint
      // Tag (1) + length varint (2) + subject (200) + id (8)
      expect(encoded.length).toBe(1 + 2 + 200 + 8);
    });
  });

  describe('u64 encoding', () => {
    it('should handle small u64 values', () => {
      const msg: ClientMessage = { type: 'Unsubscribe', id: 1 };
      const encoded = encodeClientMessage(msg);
      
      // Tag (1) + id (8)
      expect(encoded.length).toBe(9);
    });

    it('should handle large u64 values', () => {
      const msg: ClientMessage = { type: 'Unsubscribe', id: Number.MAX_SAFE_INTEGER };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded.length).toBe(9);
    });

    it('should handle zero', () => {
      const msg: ClientMessage = { type: 'Unsubscribe', id: 0 };
      const encoded = encodeClientMessage(msg);
      
      expect(encoded.length).toBe(9);
    });
  });
});

// Helper function for tests
function encodeVarInt(n: number): Uint8Array {
  const bytes: number[] = [];
  while (n >= 0x80) {
    bytes.push((n & 0x7f) | 0x80);
    n >>>= 7;
  }
  bytes.push(n);
  return new Uint8Array(bytes);
}
