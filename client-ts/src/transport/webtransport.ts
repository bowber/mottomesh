/**
 * WebTransport transport implementation
 */

import type { Transport } from './interface';

export class WebTransportTransport implements Transport {
  private transport: WebTransport | null = null;
  private writer: WritableStreamDefaultWriter<Uint8Array> | null = null;
  private messageHandler: ((data: Uint8Array) => void) | null = null;
  private closeHandler: ((reason?: string) => void) | null = null;
  // @ts-expect-error - reserved for future use
  private errorHandler: ((error: Error) => void) | null = null;

  constructor(private url: string) {}

  async connect(): Promise<void> {
    if (!('WebTransport' in globalThis)) {
      throw new Error('WebTransport is not supported in this environment');
    }

    try {
      this.transport = new WebTransport(this.url);
      
      await this.transport.ready;

      // Set up datagram reading
      void this.readDatagrams();
      
      // Set up bidirectional stream reading
      void this.readBidirectionalStreams();

      // Handle connection close
      this.transport.closed.then(() => {
        this.closeHandler?.('Connection closed');
      }).catch((error: unknown) => {
        const message = error instanceof Error ? error.message : String(error);
        this.closeHandler?.(message);
      });

    } catch (err) {
      throw new Error(`WebTransport connection failed: ${String(err)}`);
    }
  }

  async disconnect(): Promise<void> {
    if (this.writer) {
      try {
        await this.writer.close();
      } catch {
        // Writer already closed, ignore
      }
      this.writer = null;
    }
    
    if (this.transport) {
      try {
        this.transport.close();
      } catch {
        // Transport already closed, ignore
      }
      this.transport = null;
    }
  }

  async send(data: Uint8Array): Promise<void> {
    if (!this.transport) {
      throw new Error('Not connected');
    }

    // Prefer datagrams for small messages (unreliable but fast)
    // Fall back to bidirectional streams for reliability
    try {
      const writer = this.transport.datagrams.writable.getWriter();
      await writer.write(data);
      writer.releaseLock();
    } catch {
      // Fall back to reliable stream
      const stream = await this.transport.createBidirectionalStream();
      const writer = stream.writable.getWriter();
      await writer.write(data);
      await writer.close();
    }
  }

  isConnected(): boolean {
    return this.transport !== null;
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

  private async readDatagrams(): Promise<void> {
    if (!this.transport) return;

    try {
      const reader = this.transport.datagrams.readable.getReader();
      
      for (;;) {
        const result: ReadableStreamReadResult<Uint8Array> = await reader.read();
        if (result.done) break;
        this.messageHandler?.(result.value);
      }
    } catch {
      // Datagram reading ended
    }
  }

  private async readBidirectionalStreams(): Promise<void> {
    if (!this.transport) return;

    try {
      const reader = this.transport.incomingBidirectionalStreams.getReader();
      
      for (;;) {
        const result: ReadableStreamReadResult<WebTransportBidirectionalStream> = await reader.read();
        if (result.done) break;
        void this.handleIncomingStream(result.value);
      }
    } catch {
      // Stream reading ended
    }
  }

  private async handleIncomingStream(stream: WebTransportBidirectionalStream): Promise<void> {
    const reader = stream.readable.getReader();
    
    try {
      for (;;) {
        const result: ReadableStreamReadResult<Uint8Array> = await reader.read();
        if (result.done) break;
        this.messageHandler?.(result.value);
      }
    } catch {
      // Stream reading ended
    }
  }
}
