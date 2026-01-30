/**
 * Mottomesh TypeScript Client
 * 
 * A client library for connecting to Mottomesh gateway with WebTransport (preferred)
 * and WebSocket (fallback) support.
 * 
 * @example
 * ```typescript
 * import { MottomeshClient } from '@mottomesh/client';
 * 
 * const client = new MottomeshClient({
 *   url: 'https://localhost:4433',
 *   token: 'your-jwt-token',
 * });
 * 
 * await client.connect();
 * 
 * // Subscribe to messages
 * const sub = client.subscribe('messages', (msg) => {
 *   console.log('Received:', msg.subject, msg.payload);
 * });
 * 
 * // Publish a message
 * await client.publish('messages', new Uint8Array([1, 2, 3]));
 * 
 * // Clean up
 * await sub.unsubscribe();
 * await client.disconnect();
 * ```
 */

export { MottomeshClient, type ClientOptions, type Subscription, type MessageCallback, type EventType } from './client';
export { Transport, TransportType, WebTransportTransport, WebSocketTransport } from './transport';
export { ClientMessage, ServerMessage, ErrorCodes, encodeClientMessage, decodeServerMessage } from './protocol';

// Backwards compatibility export (deprecated)
// eslint-disable-next-line @typescript-eslint/no-deprecated
export { MessageCodec } from './protocol';
