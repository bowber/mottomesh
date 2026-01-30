/**
 * Binary codec for protocol messages
 * Uses a simple binary format compatible with bitcode on the Rust side
 */

import type { ClientMessage, ServerMessage } from './messages';

// Message type tags (must match Rust enum order)
const CLIENT_MSG_TAGS = {
  Auth: 0,
  Subscribe: 1,
  Unsubscribe: 2,
  Publish: 3,
  Request: 4,
  Ping: 5,
} as const;

const SERVER_MSG_TAGS = {
  AuthOk: 0,
  AuthError: 1,
  SubscribeOk: 2,
  SubscribeError: 3,
  Message: 4,
  Response: 5,
  RequestError: 6,
  Error: 7,
  Pong: 8,
} as const;

/**
 * Simple binary encoder/decoder
 * Note: This is a simplified implementation. In production, you might want to use
 * a proper bitcode implementation or generate bindings from Rust.
 */

const textEncoder = new TextEncoder();
const textDecoder = new TextDecoder();

// Encoding helpers
function encodeString(s: string): Uint8Array {
  const bytes = textEncoder.encode(s);
  const len = encodeVarInt(bytes.length);
  return concat([len, bytes]);
}

function encodeBytes(data: Uint8Array): Uint8Array {
  const len = encodeVarInt(data.length);
  return concat([len, data]);
}

function encodeU32(n: number): Uint8Array {
  const buf = new ArrayBuffer(4);
  new DataView(buf).setUint32(0, n, true); // little-endian
  return new Uint8Array(buf);
}

function encodeU64(n: number): Uint8Array {
  const buf = new ArrayBuffer(8);
  const view = new DataView(buf);
  view.setUint32(0, n & 0xffffffff, true);
  view.setUint32(4, Math.floor(n / 0x100000000), true);
  return new Uint8Array(buf);
}

function encodeVarInt(n: number): Uint8Array {
  const bytes: number[] = [];
  while (n >= 0x80) {
    bytes.push((n & 0x7f) | 0x80);
    n >>>= 7;
  }
  bytes.push(n);
  return new Uint8Array(bytes);
}

// Decoding helpers
function decodeString(data: Uint8Array, offset: number): [string, number] {
  const [len, newOffset] = decodeVarInt(data, offset);
  const str = textDecoder.decode(data.slice(newOffset, newOffset + len));
  return [str, newOffset + len];
}

function decodeBytes(data: Uint8Array, offset: number): [Uint8Array, number] {
  const [len, newOffset] = decodeVarInt(data, offset);
  return [data.slice(newOffset, newOffset + len), newOffset + len];
}

function decodeU32(data: Uint8Array, offset: number): [number, number] {
  const view = new DataView(data.buffer, data.byteOffset + offset, 4);
  return [view.getUint32(0, true), offset + 4];
}

function decodeU64(data: Uint8Array, offset: number): [number, number] {
  const view = new DataView(data.buffer, data.byteOffset + offset, 8);
  const low = view.getUint32(0, true);
  const high = view.getUint32(4, true);
  return [low + high * 0x100000000, offset + 8];
}

function decodeVarInt(data: Uint8Array, offset: number): [number, number] {
  let n = 0;
  let shift = 0;
  let i = offset;
  while (i < data.length) {
    const b = data[i++];
    n |= (b & 0x7f) << shift;
    if ((b & 0x80) === 0) break;
    shift += 7;
  }
  return [n, i];
}

function concat(parts: Uint8Array[]): Uint8Array {
  const totalLen = parts.reduce((acc, p) => acc + p.length, 0);
  const result = new Uint8Array(totalLen);
  let offset = 0;
  for (const part of parts) {
    result.set(part, offset);
    offset += part.length;
  }
  return result;
}

/**
 * Encode a client message to binary
 */
export function encodeClientMessage(msg: ClientMessage): Uint8Array {
  const parts: Uint8Array[] = [];

  switch (msg.type) {
    case 'Auth':
      parts.push(new Uint8Array([CLIENT_MSG_TAGS.Auth]));
      parts.push(encodeString(msg.token));
      break;

    case 'Subscribe':
      parts.push(new Uint8Array([CLIENT_MSG_TAGS.Subscribe]));
      parts.push(encodeString(msg.subject));
      parts.push(encodeU64(msg.id));
      break;

    case 'Unsubscribe':
      parts.push(new Uint8Array([CLIENT_MSG_TAGS.Unsubscribe]));
      parts.push(encodeU64(msg.id));
      break;

    case 'Publish':
      parts.push(new Uint8Array([CLIENT_MSG_TAGS.Publish]));
      parts.push(encodeString(msg.subject));
      parts.push(encodeBytes(msg.payload));
      break;

    case 'Request':
      parts.push(new Uint8Array([CLIENT_MSG_TAGS.Request]));
      parts.push(encodeString(msg.subject));
      parts.push(encodeBytes(msg.payload));
      parts.push(encodeU32(msg.timeoutMs));
      parts.push(encodeU64(msg.requestId));
      break;

    case 'Ping':
      parts.push(new Uint8Array([CLIENT_MSG_TAGS.Ping]));
      break;
  }

  return concat(parts);
}

/**
 * Decode a server message from binary
 */
export function decodeServerMessage(data: Uint8Array): ServerMessage {
  let offset = 0;

  const tag = data[offset++];

  switch (tag) {
    case SERVER_MSG_TAGS.AuthOk: {
      const [sessionId] = decodeString(data, offset);
      return { type: 'AuthOk', sessionId };
    }

    case SERVER_MSG_TAGS.AuthError: {
      const [reason] = decodeString(data, offset);
      return { type: 'AuthError', reason };
    }

    case SERVER_MSG_TAGS.SubscribeOk: {
      const [id] = decodeU64(data, offset);
      return { type: 'SubscribeOk', id };
    }

    case SERVER_MSG_TAGS.SubscribeError: {
      const [id, offset2] = decodeU64(data, offset);
      const [reason] = decodeString(data, offset2);
      return { type: 'SubscribeError', id, reason };
    }

    case SERVER_MSG_TAGS.Message: {
      const [subscriptionId, offset2] = decodeU64(data, offset);
      const [subject, offset3] = decodeString(data, offset2);
      const [payload] = decodeBytes(data, offset3);
      return { type: 'Message', subscriptionId, subject, payload };
    }

    case SERVER_MSG_TAGS.Response: {
      const [requestId, offset2] = decodeU64(data, offset);
      const [payload] = decodeBytes(data, offset2);
      return { type: 'Response', requestId, payload };
    }

    case SERVER_MSG_TAGS.RequestError: {
      const [requestId, offset2] = decodeU64(data, offset);
      const [reason] = decodeString(data, offset2);
      return { type: 'RequestError', requestId, reason };
    }

    case SERVER_MSG_TAGS.Error: {
      const [code, offset2] = decodeU32(data, offset);
      const [message] = decodeString(data, offset2);
      return { type: 'Error', code, message };
    }

    case SERVER_MSG_TAGS.Pong:
      return { type: 'Pong' };

    default:
      throw new Error(`Unknown server message tag: ${tag}`);
  }
}

/**
 * @deprecated Use encodeClientMessage and decodeServerMessage instead
 */
export const MessageCodec = {
  encodeClient: encodeClientMessage,
  decodeServer: decodeServerMessage,
};
