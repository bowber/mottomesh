/**
 * Protocol message types for Mottomesh gateway communication
 * These mirror the Rust protocol definitions in crates/gateway/src/protocol/messages.rs
 */

// Client -> Server messages
export type ClientMessage =
  | { type: 'Auth'; token: string }
  | { type: 'Subscribe'; subject: string; id: number }
  | { type: 'Unsubscribe'; id: number }
  | { type: 'Publish'; subject: string; payload: Uint8Array }
  | { type: 'Request'; subject: string; payload: Uint8Array; timeoutMs: number; requestId: number }
  | { type: 'Ping' };

// Server -> Client messages
export type ServerMessage =
  | { type: 'AuthOk'; sessionId: string }
  | { type: 'AuthError'; reason: string }
  | { type: 'SubscribeOk'; id: number }
  | { type: 'SubscribeError'; id: number; reason: string }
  | { type: 'Message'; subscriptionId: number; subject: string; payload: Uint8Array }
  | { type: 'Response'; requestId: number; payload: Uint8Array }
  | { type: 'RequestError'; requestId: number; reason: string }
  | { type: 'Error'; code: number; message: string }
  | { type: 'Pong' };

// Error codes (matching Rust definitions)
export const ErrorCodes = {
  UNAUTHORIZED: 401,
  FORBIDDEN: 403,
  NOT_FOUND: 404,
  INTERNAL_ERROR: 500,
  INVALID_MESSAGE: 400,
} as const;
