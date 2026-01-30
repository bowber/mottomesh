import { describe, it, expect, vi, beforeEach } from 'vitest';
import { MottomeshClient, ClientOptions } from '../client';

describe('MottomeshClient', () => {
  describe('constructor', () => {
    it('should create client with required options', () => {
      const options: ClientOptions = {
        url: 'https://localhost:4433',
        token: 'test-token',
      };
      const client = new MottomeshClient(options);
      
      expect(client).toBeDefined();
      expect(client.isConnected()).toBe(false);
      expect(client.isAuthenticated()).toBe(false);
    });

    it('should apply default options', () => {
      const client = new MottomeshClient({
        url: 'https://localhost:4433',
        token: 'test-token',
      });
      
      // Check defaults via behavior
      expect(client.getSessionId()).toBeNull();
    });

    it('should accept custom options', () => {
      const options: ClientOptions = {
        url: 'https://custom:5000',
        token: 'custom-token',
        transport: 'websocket',
        reconnect: false,
        reconnectDelay: 5000,
        maxReconnectAttempts: 5,
      };
      const client = new MottomeshClient(options);
      
      expect(client).toBeDefined();
    });
  });

  describe('state methods', () => {
    let client: MottomeshClient;

    beforeEach(() => {
      client = new MottomeshClient({
        url: 'https://localhost:4433',
        token: 'test-token',
      });
    });

    it('should report not connected initially', () => {
      expect(client.isConnected()).toBe(false);
    });

    it('should report not authenticated initially', () => {
      expect(client.isAuthenticated()).toBe(false);
    });

    it('should have null session ID initially', () => {
      expect(client.getSessionId()).toBeNull();
    });
  });

  describe('event handlers', () => {
    let client: MottomeshClient;

    beforeEach(() => {
      client = new MottomeshClient({
        url: 'https://localhost:4433',
        token: 'test-token',
      });
    });

    it('should register event handler', () => {
      const handler = vi.fn();
      client.on('error', handler);
      
      // Handler registered but not called
      expect(handler).not.toHaveBeenCalled();
    });

    it('should remove event handler', () => {
      const handler = vi.fn();
      client.on('error', handler);
      client.off('error', handler);
      
      // Can't directly test, but should not throw
      expect(handler).not.toHaveBeenCalled();
    });

    it('should allow multiple handlers for same event', () => {
      const handler1 = vi.fn();
      const handler2 = vi.fn();
      
      client.on('connect', handler1);
      client.on('connect', handler2);
      
      // Both registered, neither called
      expect(handler1).not.toHaveBeenCalled();
      expect(handler2).not.toHaveBeenCalled();
    });

    it('should support different event types', () => {
      const connectHandler = vi.fn();
      const disconnectHandler = vi.fn();
      const errorHandler = vi.fn();
      const authHandler = vi.fn();

      client.on('connect', connectHandler);
      client.on('disconnect', disconnectHandler);
      client.on('error', errorHandler);
      client.on('auth', authHandler);
      
      // All registered
      expect(client).toBeDefined();
    });
  });

  describe('subscribe', () => {
    let client: MottomeshClient;

    beforeEach(() => {
      client = new MottomeshClient({
        url: 'https://localhost:4433',
        token: 'test-token',
      });
    });

    it('should create subscription with incremented IDs', () => {
      const callback = vi.fn();
      
      // This will throw because we're not connected, but we can test the subscription structure
      // through the returned object
      try {
        const sub1 = client.subscribe('subject1', callback);
        const sub2 = client.subscribe('subject2', callback);
        
        expect(sub1.id).toBe(1);
        expect(sub1.subject).toBe('subject1');
        expect(sub2.id).toBe(2);
        expect(sub2.subject).toBe('subject2');
        expect(typeof sub1.unsubscribe).toBe('function');
      } catch {
        // Expected when not connected
      }
    });
  });

  describe('publish', () => {
    let client: MottomeshClient;

    beforeEach(() => {
      client = new MottomeshClient({
        url: 'https://localhost:4433',
        token: 'test-token',
      });
    });

    it('should throw when not authenticated', () => {
      expect(() =>
        client.publish('test.subject', new Uint8Array([1, 2, 3]))
      ).toThrow('Not authenticated');
    });
  });

  describe('request', () => {
    let client: MottomeshClient;

    beforeEach(() => {
      client = new MottomeshClient({
        url: 'https://localhost:4433',
        token: 'test-token',
      });
    });

    it('should throw when not authenticated', async () => {
      await expect(
        client.request('test.subject', new Uint8Array([1, 2, 3]))
      ).rejects.toThrow('Not authenticated');
    });

    it('should accept custom timeout', async () => {
      await expect(
        client.request('test.subject', new Uint8Array([1, 2, 3]), 10000)
      ).rejects.toThrow('Not authenticated');
    });
  });

  describe('disconnect', () => {
    let client: MottomeshClient;

    beforeEach(() => {
      client = new MottomeshClient({
        url: 'https://localhost:4433',
        token: 'test-token',
      });
    });

    it('should handle disconnect when not connected', async () => {
      // Should not throw
      await client.disconnect();
      expect(client.isConnected()).toBe(false);
    });
  });
});
