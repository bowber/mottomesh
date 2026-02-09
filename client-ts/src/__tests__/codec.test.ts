import { describe, expect, it } from 'vitest';
import { decodeClientEnvelope, encodeServerEnvelope } from '@motto/schema';

import { decodeServerMessage, encodeClientMessage } from '../protocol/codec';
import type { ClientMessage } from '../protocol/messages';

describe('MessageCodec', () => {
  it('encodes auth messages with schema envelope', () => {
    const msg: ClientMessage = { type: 'Auth', token: 'test.jwt.token' };
    const encoded = encodeClientMessage(msg);
    const decoded = decodeClientEnvelope(encoded);

    expect(decoded.message.type).toBe('Auth');
    if (decoded.message.type === 'Auth') {
      expect(decoded.message.token).toBe('test.jwt.token');
    }
  });

  it('encodes request messages with field mapping', () => {
    const msg: ClientMessage = {
      type: 'Request',
      subject: 'api.test',
      payload: new Uint8Array([1, 2, 3]),
      timeoutMs: 5000,
      requestId: 99,
    };

    const encoded = encodeClientMessage(msg);
    const decoded = decodeClientEnvelope(encoded);

    expect(decoded.message.type).toBe('Request');
    if (decoded.message.type === 'Request') {
      expect(decoded.message.timeout_ms).toBe(5000);
      expect(decoded.message.request_id).toBe(99n);
      expect(decoded.message.payload).toEqual([1, 2, 3]);
    }
  });

  it('decodes server message payloads into Uint8Array', () => {
    const encoded = encodeServerEnvelope({
      message: {
        type: 'Message',
        subscription_id: 42n,
        subject: 'messages',
        payload: [9, 8, 7],
      },
    });

    const decoded = decodeServerMessage(encoded);
    expect(decoded.type).toBe('Message');
    if (decoded.type === 'Message') {
      expect(decoded.subscriptionId).toBe(42);
      expect(decoded.payload).toEqual(new Uint8Array([9, 8, 7]));
    }
  });

  it('throws for unsafe integer ids', () => {
    const encoded = encodeServerEnvelope({
      message: {
        type: 'SubscribeOk',
        id: BigInt(Number.MAX_SAFE_INTEGER) + 1n,
      },
    });

    expect(() => decodeServerMessage(encoded)).toThrow('safe integer range');
  });
});
