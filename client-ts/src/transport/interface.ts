/**
 * Transport interface for Mottomesh client
 * Abstracts over WebTransport and WebSocket
 */

export interface Transport {
  /** Connect to the server */
  connect(): Promise<void>;
  
  /** Disconnect from the server */
  disconnect(): Promise<void>;
  
  /** Send binary data */
  send(data: Uint8Array): Promise<void>;
  
  /** Check if connected */
  isConnected(): boolean;
  
  /** Register message handler */
  onMessage(handler: (data: Uint8Array) => void): void;
  
  /** Register close handler */
  onClose(handler: (reason?: string) => void): void;
  
  /** Register error handler */
  onError(handler: (error: Error) => void): void;
}

export type TransportType = 'auto' | 'webtransport' | 'websocket';
