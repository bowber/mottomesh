/**
 * Main Mottomesh client
 */

import { Transport, TransportType, WebTransportTransport, WebSocketTransport } from './transport';
import { encodeClientMessage, decodeServerMessage, ClientMessage, ServerMessage } from './protocol';

export interface ClientOptions {
  /** Gateway URL (e.g., "https://localhost:4433") */
  url: string;
  /** JWT authentication token */
  token: string;
  /** Transport type: 'auto' (default), 'webtransport', or 'websocket' */
  transport?: TransportType;
  /** Auto-reconnect on disconnect */
  reconnect?: boolean;
  /** Reconnect delay in ms (default: 1000) */
  reconnectDelay?: number;
  /** Max reconnect attempts (default: 10) */
  maxReconnectAttempts?: number;
}

export interface Subscription {
  /** Subscription ID */
  id: number;
  /** Subject pattern */
  subject: string;
  /** Unsubscribe from this subscription */
  unsubscribe(): void;
}

export type MessageCallback = (msg: {
  subject: string;
  payload: Uint8Array;
}) => void;

export type EventType = 'connect' | 'disconnect' | 'error' | 'auth';

type EventCallback = (data?: unknown) => void;

export class MottomeshClient {
  private transport: Transport | null = null;
  private options: Required<ClientOptions>;
  private authenticated = false;
  private sessionId: string | null = null;
  private nextSubId = 1;
  private nextRequestId = 1;
  private subscriptions = new Map<number, { subject: string; callback: MessageCallback }>();
  private pendingRequests = new Map<number, { resolve: (data: Uint8Array) => void; reject: (error: Error) => void }>();
  private eventHandlers = new Map<EventType, Set<EventCallback>>();
  private reconnectAttempts = 0;
  private isReconnecting = false;

  constructor(options: ClientOptions) {
    this.options = {
      url: options.url,
      token: options.token,
      transport: options.transport ?? 'auto',
      reconnect: options.reconnect ?? true,
      reconnectDelay: options.reconnectDelay ?? 1000,
      maxReconnectAttempts: options.maxReconnectAttempts ?? 10,
    };
  }

  /**
   * Connect to the gateway
   */
  async connect(): Promise<void> {
    this.transport = await this.createTransport();

    this.transport.onMessage((data) => { this.handleMessage(data); });
    this.transport.onClose((reason) => { this.handleClose(reason); });
    this.transport.onError((error) => { this.emit('error', error); });

    await this.transport.connect();

    // Authenticate with the gateway
    await this.authenticate();

    this.reconnectAttempts = 0;
    this.emit('connect');
  }

  /**
   * Disconnect from the gateway
   */
  async disconnect(): Promise<void> {
    this.options.reconnect = false; // Prevent auto-reconnect
    if (this.transport) {
      await this.transport.disconnect();
      this.transport = null;
    }
    this.authenticated = false;
    this.sessionId = null;
  }

  /**
   * Subscribe to a subject
   */
  subscribe(subject: string, callback: MessageCallback): Subscription {
    const id = this.nextSubId++;

    this.subscriptions.set(id, { subject, callback });

    // Send subscribe message
    this.sendMessage({ type: 'Subscribe', subject, id });

    return {
      id,
      subject,
      unsubscribe: (): void => {
        this.subscriptions.delete(id);
        this.sendMessage({ type: 'Unsubscribe', id });
      },
    };
  }

  /**
   * Publish a message to a subject
   */
  publish(subject: string, payload: Uint8Array): void {
    if (!this.authenticated) {
      throw new Error('Not authenticated');
    }
    this.sendMessage({ type: 'Publish', subject, payload });
  }

  /**
   * Request-reply pattern
   */
  async request(subject: string, payload: Uint8Array, timeout = 5000): Promise<Uint8Array> {
    if (!this.authenticated) {
      throw new Error('Not authenticated');
    }

    const requestId = this.nextRequestId++;

    return new Promise((resolve, reject) => {
      // Set up timeout
      const timer = setTimeout(() => {
        this.pendingRequests.delete(requestId);
        reject(new Error('Request timeout'));
      }, timeout);

      this.pendingRequests.set(requestId, {
        resolve: (data): void => {
          clearTimeout(timer);
          resolve(data);
        },
        reject: (error): void => {
          clearTimeout(timer);
          reject(error);
        },
      });

      this.sendMessage({
        type: 'Request',
        subject,
        payload,
        timeoutMs: timeout,
        requestId,
      });
    });
  }

  /**
   * Check if connected and authenticated
   */
  isConnected(): boolean {
    return this.transport?.isConnected() ?? false;
  }

  /**
   * Check if authenticated
   */
  isAuthenticated(): boolean {
    return this.authenticated;
  }

  /**
   * Get session ID
   */
  getSessionId(): string | null {
    return this.sessionId;
  }

  /**
   * Register event handler
   */
  on(event: EventType, callback: EventCallback): void {
    let handlers = this.eventHandlers.get(event);
    if (!handlers) {
      handlers = new Set();
      this.eventHandlers.set(event, handlers);
    }
    handlers.add(callback);
  }

  /**
   * Remove event handler
   */
  off(event: EventType, callback: EventCallback): void {
    this.eventHandlers.get(event)?.delete(callback);
  }

  // Private methods

  private async createTransport(): Promise<Transport> {
    const baseUrl = this.options.url;

    if (this.options.transport === 'webtransport') {
      return new WebTransportTransport(baseUrl);
    }

    if (this.options.transport === 'websocket') {
      const wsUrl = this.getWebSocketUrl(baseUrl);
      return new WebSocketTransport(wsUrl);
    }

    // Auto-detect: try WebTransport first, fall back to WebSocket
    if ('WebTransport' in globalThis) {
      try {
        const wt = new WebTransportTransport(baseUrl);
        await wt.connect();
        return wt;
      } catch {
        console.warn('WebTransport connection failed, falling back to WebSocket');
      }
    }

    const wsUrl = this.getWebSocketUrl(baseUrl);
    return new WebSocketTransport(wsUrl);
  }

  private getWebSocketUrl(baseUrl: string): string {
    // Convert HTTPS URL to WebSocket URL
    // The WebSocket server runs on port + 1
    const url = new URL(baseUrl);
    const port = parseInt(url.port || '443', 10) + 1;
    const protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
    return `${protocol}//${url.hostname}:${port}/ws`;
  }

  private async authenticate(): Promise<void> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Authentication timeout'));
      }, 10000);

      const originalHandler = this.handleMessage.bind(this);

      // Temporarily override message handler to catch auth response
      const authHandler = (data: Uint8Array): void => {
        const msg = decodeServerMessage(data);

        if (msg.type === 'AuthOk') {
          clearTimeout(timeout);
          this.authenticated = true;
          this.sessionId = msg.sessionId;
          this.emit('auth', { sessionId: msg.sessionId });
          resolve();
        } else if (msg.type === 'AuthError') {
          clearTimeout(timeout);
          reject(new Error(`Authentication failed: ${msg.reason}`));
        } else {
          // Pass to normal handler
          originalHandler(data);
        }
      };

      if (this.transport) {
        this.transport.onMessage(authHandler);
      }

      // Send auth message
      this.sendMessage({ type: 'Auth', token: this.options.token });
    });
  }

  private handleMessage(data: Uint8Array): void {
    let msg: ServerMessage;
    try {
      msg = decodeServerMessage(data);
    } catch (error) {
      console.error('Failed to decode server message:', error);
      return;
    }

    switch (msg.type) {
      case 'Message': {
        const sub = this.subscriptions.get(msg.subscriptionId);
        if (sub) {
          sub.callback({ subject: msg.subject, payload: msg.payload });
        }
        break;
      }

      case 'Response': {
        const pending = this.pendingRequests.get(msg.requestId);
        if (pending) {
          this.pendingRequests.delete(msg.requestId);
          pending.resolve(msg.payload);
        }
        break;
      }

      case 'RequestError': {
        const pending = this.pendingRequests.get(msg.requestId);
        if (pending) {
          this.pendingRequests.delete(msg.requestId);
          pending.reject(new Error(msg.reason));
        }
        break;
      }

      case 'SubscribeOk':
        // Subscription confirmed
        break;

      case 'SubscribeError':
        console.error(`Subscription error for id ${msg.id}: ${msg.reason}`);
        this.subscriptions.delete(msg.id);
        break;

      case 'Error':
        this.emit('error', new Error(`Server error ${msg.code}: ${msg.message}`));
        break;

      case 'Pong':
        // Keepalive response
        break;
    }
  }

  private handleClose(reason?: string): void {
    this.authenticated = false;
    this.sessionId = null;
    this.emit('disconnect', reason);

    // Auto-reconnect if enabled
    if (this.options.reconnect && !this.isReconnecting) {
      void this.attemptReconnect();
    }
  }

  private async attemptReconnect(): Promise<void> {
    if (this.reconnectAttempts >= this.options.maxReconnectAttempts) {
      console.error('Max reconnect attempts reached');
      return;
    }

    this.isReconnecting = true;
    this.reconnectAttempts++;

    console.log(`Attempting reconnect (${this.reconnectAttempts}/${this.options.maxReconnectAttempts})...`);

    await new Promise((resolve) => setTimeout(resolve, this.options.reconnectDelay));

    try {
      await this.connect();

      // Resubscribe to all subjects
      for (const [id, { subject }] of this.subscriptions) {
        this.sendMessage({ type: 'Subscribe', subject, id });
      }

      this.isReconnecting = false;
    } catch (error) {
      console.error('Reconnect failed:', error);
      this.isReconnecting = false;
      void this.attemptReconnect();
    }
  }

  private sendMessage(msg: ClientMessage): void {
    if (!this.transport) {
      throw new Error('Not connected');
    }
    const encoded = encodeClientMessage(msg);
    void this.transport.send(encoded);
  }

  private emit(event: EventType, data?: unknown): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(data);
        } catch (error) {
          console.error(`Error in ${event} handler:`, error);
        }
      }
    }
  }
}
