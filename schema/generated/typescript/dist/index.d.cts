type ClientMessage = {
    type: 'Auth';
    token: string;
} | {
    type: 'Subscribe';
    subject: string;
    id: bigint;
} | {
    type: 'Unsubscribe';
    id: bigint;
} | {
    type: 'Publish';
    subject: string;
    payload: number[];
} | {
    type: 'Request';
    subject: string;
    payload: number[];
    timeout_ms: number;
    request_id: bigint;
} | {
    type: 'Ping';
};
type ServerMessage = {
    type: 'AuthOk';
    session_id: string;
} | {
    type: 'AuthError';
    reason: string;
} | {
    type: 'SubscribeOk';
    id: bigint;
} | {
    type: 'SubscribeError';
    id: bigint;
    reason: string;
} | {
    type: 'Message';
    subscription_id: bigint;
    subject: string;
    payload: number[];
} | {
    type: 'Response';
    request_id: bigint;
    payload: number[];
} | {
    type: 'RequestError';
    request_id: bigint;
    reason: string;
} | {
    type: 'Error';
    code: number;
    message: string;
} | {
    type: 'Pong';
};
interface InnerData {
    id: number[];
    name: string[];
}
interface TestData {
    id: number;
    name: string;
    inner_data: InnerData;
}
interface ClientEnvelope {
    message: ClientMessage;
}
interface ServerEnvelope {
    message: ServerMessage;
}
/**
 * Auto-generated router enum for schema schema.
 *
 * This enum wraps all message types for type-safe routing.
 */
type SchemaRouter = {
    type: 'InnerData';
    data: InnerData;
} | {
    type: 'TestData';
    data: TestData;
} | {
    type: 'ClientEnvelope';
    data: ClientEnvelope;
} | {
    type: 'ServerEnvelope';
    data: ServerEnvelope;
};
/** Message type discriminants for SchemaRouter */
declare const SchemaRouterType: {
    readonly InnerData: 0;
    readonly TestData: 1;
    readonly ClientEnvelope: 2;
    readonly ServerEnvelope: 3;
};

declare const PROTOCOL_VERSION_BYTE = 109;
declare const SCHEMA_FINGERPRINT = "6dd935d3b48ac8c177ad5995ff3060d02268379a59ca971c171bd881acb2f35a";
/** Zero-copy buffer view for packet framing */
declare class PacketView {
    private view;
    private offset;
    constructor(buffer: ArrayBuffer | Uint8Array);
    /** Get protocol version byte from packet header */
    getVersionByte(): number;
    /** Validate version matches expected */
    validateVersion(): boolean;
    /** Read u8 at current offset */
    readU8(): number;
    /** Read u16 (little-endian) */
    readU16(): number;
    /** Read u32 (little-endian) */
    readU32(): number;
    /** Read u64 as BigInt (little-endian) */
    readU64(): bigint;
    /** Read f32 (little-endian) */
    readF32(): number;
    /** Read f64 (little-endian) */
    readF64(): number;
    /** Read length-prefixed string (u32 length + UTF-8 bytes) */
    readString(): string;
    /** Read boolean */
    readBool(): boolean;
    /** Skip bytes */
    skip(n: number): void;
    /** Get current offset */
    getOffset(): number;
    /** Set offset */
    setOffset(offset: number): void;
    /** Get remaining bytes */
    remaining(): number;
}
/** Packet builder for zero-copy encoding */
declare class PacketBuilder {
    private buffer;
    private view;
    private offset;
    constructor(initialSize?: number);
    private ensureCapacity;
    writeU8(val: number): void;
    writeU16(val: number): void;
    writeU32(val: number): void;
    writeU64(val: bigint): void;
    writeF32(val: number): void;
    writeF64(val: number): void;
    writeString(val: string): void;
    writeBool(val: boolean): void;
    /** Get the built packet as a Uint8Array (trimmed to actual size) */
    build(): Uint8Array;
}
/** Encode InnerData to binary */
declare function encodeInnerData(msg: InnerData): Uint8Array;
/** Decode InnerData from binary */
declare function decodeInnerData(data: Uint8Array): InnerData;
/** Encode TestData to binary */
declare function encodeTestData(msg: TestData): Uint8Array;
/** Decode TestData from binary */
declare function decodeTestData(data: Uint8Array): TestData;
/** Encode ClientEnvelope to binary */
declare function encodeClientEnvelope(msg: ClientEnvelope): Uint8Array;
/** Decode ClientEnvelope from binary */
declare function decodeClientEnvelope(data: Uint8Array): ClientEnvelope;
/** Encode ServerEnvelope to binary */
declare function encodeServerEnvelope(msg: ServerEnvelope): Uint8Array;
/** Decode ServerEnvelope from binary */
declare function decodeServerEnvelope(data: Uint8Array): ServerEnvelope;

declare const PROTOCOL_VERSION = 109;
/** Connection state machine */
declare enum ConnectionState {
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
    Reconnecting = 3,
    Error = 4
}
/** Retry configuration */
interface RetryConfig {
    maxRetries: number;
    initialDelayMs: number;
    maxDelayMs: number;
    backoffMultiplier: number;
}
declare const DEFAULT_RETRY_CONFIG: RetryConfig;
/** Calculate retry delay with exponential backoff */
declare function calculateRetryDelay(attempt: number, config?: RetryConfig): number;
/** Decompress zstd data (requires external zstd library) */
declare function decompressZstd(data: Uint8Array): Promise<Uint8Array>;
/** Compress data with zstd (requires external zstd library) */
declare function compressZstd(data: Uint8Array, level?: number): Promise<Uint8Array>;
/** WebTransport connection wrapper */
declare class MottoTransport {
    private url;
    private transport;
    private state;
    private retryAttempt;
    private retryConfig;
    constructor(url: string, retryConfig?: RetryConfig);
    connect(): Promise<void>;
    reconnect(): Promise<void>;
    sendDatagram(data: Uint8Array): Promise<void>;
    receiveDatagram(): AsyncGenerator<Uint8Array>;
    getState(): ConnectionState;
    close(): Promise<void>;
}

export { type ClientEnvelope, type ClientMessage, ConnectionState, DEFAULT_RETRY_CONFIG, type InnerData, MottoTransport, PROTOCOL_VERSION, PROTOCOL_VERSION_BYTE, PacketBuilder, PacketView, type RetryConfig, SCHEMA_FINGERPRINT, type SchemaRouter, SchemaRouterType, type ServerEnvelope, type ServerMessage, type TestData, calculateRetryDelay, compressZstd, decodeClientEnvelope, decodeInnerData, decodeServerEnvelope, decodeTestData, decompressZstd, encodeClientEnvelope, encodeInnerData, encodeServerEnvelope, encodeTestData };
