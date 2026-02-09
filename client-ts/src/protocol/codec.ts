import {
  decodeServerEnvelope,
  encodeClientEnvelope,
  type ClientMessage as SchemaClientMessage,
  type ServerMessage as SchemaServerMessage,
} from '@motto/schema';
import type { ClientMessage, ServerMessage } from './messages';

function toBigIntId(value: number): bigint {
  return BigInt(value);
}

function toNumberId(value: bigint): number {
  const num = Number(value);
  if (!Number.isSafeInteger(num)) {
    throw new Error(`ID exceeds JS safe integer range: ${value.toString()}`);
  }
  return num;
}

function toSchemaClientMessage(msg: ClientMessage): SchemaClientMessage {
  switch (msg.type) {
    case 'Auth':
      return { type: 'Auth', token: msg.token };
    case 'Subscribe':
      return { type: 'Subscribe', subject: msg.subject, id: toBigIntId(msg.id) };
    case 'Unsubscribe':
      return { type: 'Unsubscribe', id: toBigIntId(msg.id) };
    case 'Publish':
      return { type: 'Publish', subject: msg.subject, payload: Array.from(msg.payload) };
    case 'Request':
      return {
        type: 'Request',
        subject: msg.subject,
        payload: Array.from(msg.payload),
        timeout_ms: msg.timeoutMs,
        request_id: toBigIntId(msg.requestId),
      };
    case 'Ping':
      return { type: 'Ping' };
  }
}

function toPublicServerMessage(msg: SchemaServerMessage): ServerMessage {
  switch (msg.type) {
    case 'AuthOk':
      return { type: 'AuthOk', sessionId: msg.session_id };
    case 'AuthError':
      return { type: 'AuthError', reason: msg.reason };
    case 'SubscribeOk':
      return { type: 'SubscribeOk', id: toNumberId(msg.id) };
    case 'SubscribeError':
      return { type: 'SubscribeError', id: toNumberId(msg.id), reason: msg.reason };
    case 'Message':
      return {
        type: 'Message',
        subscriptionId: toNumberId(msg.subscription_id),
        subject: msg.subject,
        payload: new Uint8Array(msg.payload),
      };
    case 'Response':
      return {
        type: 'Response',
        requestId: toNumberId(msg.request_id),
        payload: new Uint8Array(msg.payload),
      };
    case 'RequestError':
      return {
        type: 'RequestError',
        requestId: toNumberId(msg.request_id),
        reason: msg.reason,
      };
    case 'Error':
      return { type: 'Error', code: msg.code, message: msg.message };
    case 'Pong':
      return { type: 'Pong' };
  }
}

export function encodeClientMessage(msg: ClientMessage): Uint8Array {
  return encodeClientEnvelope({ message: toSchemaClientMessage(msg) });
}

export function decodeServerMessage(data: Uint8Array): ServerMessage {
  const envelope = decodeServerEnvelope(data);
  return toPublicServerMessage(envelope.message);
}

/**
 * @deprecated Use encodeClientMessage and decodeServerMessage instead
 */
export const MessageCodec = {
  encodeClient: encodeClientMessage,
  decodeServer: decodeServerMessage,
};
