/**
 * WebSocket transport implementation
 */

import type { Transport } from './interface';

export class WebSocketTransport implements Transport {
  private socket: WebSocket | null = null;
  private messageHandler: ((data: Uint8Array) => void) | null = null;
  private closeHandler: ((reason?: string) => void) | null = null;
  private errorHandler: ((error: Error) => void) | null = null;
  private connectResolve: (() => void) | null = null;
  private connectReject: ((error: Error) => void) | null = null;

  constructor(private url: string) {}

  async connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      this.connectResolve = resolve;
      this.connectReject = reject;

      try {
        this.socket = new WebSocket(this.url);
        this.socket.binaryType = 'arraybuffer';

        this.socket.onopen = (): void => {
          this.connectResolve?.();
          this.connectResolve = null;
          this.connectReject = null;
        };

        this.socket.onmessage = (event: MessageEvent): void => {
          if (event.data instanceof ArrayBuffer) {
            this.messageHandler?.(new Uint8Array(event.data));
          }
        };

        this.socket.onclose = (event: CloseEvent): void => {
          if (this.connectReject) {
            this.connectReject(new Error(`WebSocket closed: ${event.reason || 'Unknown reason'}`));
            this.connectResolve = null;
            this.connectReject = null;
          }
          this.closeHandler?.(event.reason || undefined);
        };

        this.socket.onerror = (): void => {
          const error = new Error('WebSocket error');
          if (this.connectReject) {
            this.connectReject(error);
            this.connectResolve = null;
            this.connectReject = null;
          }
          this.errorHandler?.(error);
        };
      } catch (err) {
        reject(err instanceof Error ? err : new Error(String(err)));
      }
    });
  }

  disconnect(): Promise<void> {
    if (this.socket) {
      this.socket.close(1000, 'Client disconnect');
      this.socket = null;
    }
    return Promise.resolve();
  }

  send(data: Uint8Array): Promise<void> {
    if (!this.socket?.readyState || this.socket.readyState !== WebSocket.OPEN) {
      throw new Error('Not connected');
    }
    this.socket.send(data);
    return Promise.resolve();
  }

  isConnected(): boolean {
    return this.socket !== null && this.socket.readyState === WebSocket.OPEN;
  }

  onMessage(handler: (data: Uint8Array) => void): void {
    this.messageHandler = handler;
  }

  onClose(handler: (reason?: string) => void): void {
    this.closeHandler = handler;
  }

  onError(handler: (error: Error) => void): void {
    this.errorHandler = handler;
  }
}
