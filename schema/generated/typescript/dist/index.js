// src/types.ts
var SchemaRouterType = {
  InnerData: 0,
  TestData: 1,
  ClientEnvelope: 2,
  ServerEnvelope: 3
};

// src/codec.ts
var PROTOCOL_VERSION_BYTE = 109;
var SCHEMA_FINGERPRINT = "6dd935d3b48ac8c177ad5995ff3060d02268379a59ca971c171bd881acb2f35a";
var PacketView = class {
  view;
  offset = 0;
  constructor(buffer) {
    if (buffer instanceof Uint8Array) {
      this.view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    } else {
      this.view = new DataView(buffer);
    }
  }
  /** Get protocol version byte from packet header */
  getVersionByte() {
    return this.view.getUint8(0);
  }
  /** Validate version matches expected */
  validateVersion() {
    return this.getVersionByte() === PROTOCOL_VERSION_BYTE;
  }
  /** Read u8 at current offset */
  readU8() {
    const val = this.view.getUint8(this.offset);
    this.offset += 1;
    return val;
  }
  /** Read u16 (little-endian) */
  readU16() {
    const val = this.view.getUint16(this.offset, true);
    this.offset += 2;
    return val;
  }
  /** Read u32 (little-endian) */
  readU32() {
    const val = this.view.getUint32(this.offset, true);
    this.offset += 4;
    return val;
  }
  /** Read u64 as BigInt (little-endian) */
  readU64() {
    const val = this.view.getBigUint64(this.offset, true);
    this.offset += 8;
    return val;
  }
  /** Read f32 (little-endian) */
  readF32() {
    const val = this.view.getFloat32(this.offset, true);
    this.offset += 4;
    return val;
  }
  /** Read f64 (little-endian) */
  readF64() {
    const val = this.view.getFloat64(this.offset, true);
    this.offset += 8;
    return val;
  }
  /** Read length-prefixed string (u32 length + UTF-8 bytes) */
  readString() {
    const len = this.readU32();
    const bytes = new Uint8Array(this.view.buffer, this.view.byteOffset + this.offset, len);
    this.offset += len;
    return new TextDecoder().decode(bytes);
  }
  /** Read boolean */
  readBool() {
    return this.readU8() !== 0;
  }
  /** Skip bytes */
  skip(n) {
    this.offset += n;
  }
  /** Get current offset */
  getOffset() {
    return this.offset;
  }
  /** Set offset */
  setOffset(offset) {
    this.offset = offset;
  }
  /** Get remaining bytes */
  remaining() {
    return this.view.byteLength - this.offset;
  }
};
var PacketBuilder = class {
  buffer;
  view;
  offset = 0;
  constructor(initialSize = 256) {
    this.buffer = new Uint8Array(initialSize);
    this.view = new DataView(this.buffer.buffer);
    this.writeU8(PROTOCOL_VERSION_BYTE);
  }
  ensureCapacity(need) {
    if (this.offset + need > this.buffer.length) {
      const newSize = Math.max(this.buffer.length * 2, this.offset + need);
      const newBuffer = new Uint8Array(newSize);
      newBuffer.set(this.buffer);
      this.buffer = newBuffer;
      this.view = new DataView(this.buffer.buffer);
    }
  }
  writeU8(val) {
    this.ensureCapacity(1);
    this.view.setUint8(this.offset, val);
    this.offset += 1;
  }
  writeU16(val) {
    this.ensureCapacity(2);
    this.view.setUint16(this.offset, val, true);
    this.offset += 2;
  }
  writeU32(val) {
    this.ensureCapacity(4);
    this.view.setUint32(this.offset, val, true);
    this.offset += 4;
  }
  writeU64(val) {
    this.ensureCapacity(8);
    this.view.setBigUint64(this.offset, val, true);
    this.offset += 8;
  }
  writeF32(val) {
    this.ensureCapacity(4);
    this.view.setFloat32(this.offset, val, true);
    this.offset += 4;
  }
  writeF64(val) {
    this.ensureCapacity(8);
    this.view.setFloat64(this.offset, val, true);
    this.offset += 8;
  }
  writeString(val) {
    const bytes = new TextEncoder().encode(val);
    this.writeU32(bytes.length);
    this.ensureCapacity(bytes.length);
    this.buffer.set(bytes, this.offset);
    this.offset += bytes.length;
  }
  writeBool(val) {
    this.writeU8(val ? 1 : 0);
  }
  /** Get the built packet as a Uint8Array (trimmed to actual size) */
  build() {
    return this.buffer.slice(0, this.offset);
  }
};
function encodeClientMessageFields(val, builder) {
  switch (val.type) {
    case "Auth":
      builder.writeU8(0);
      builder.writeString(val.token);
      break;
    case "Subscribe":
      builder.writeU8(1);
      builder.writeString(val.subject);
      builder.writeU64(BigInt(val.id));
      break;
    case "Unsubscribe":
      builder.writeU8(2);
      builder.writeU64(BigInt(val.id));
      break;
    case "Publish":
      builder.writeU8(3);
      builder.writeString(val.subject);
      {
        builder.writeU32(val.payload.length);
        for (const item of val.payload) {
          builder.writeU8(item);
        }
      }
      ;
      break;
    case "Request":
      builder.writeU8(4);
      builder.writeString(val.subject);
      {
        builder.writeU32(val.payload.length);
        for (const item of val.payload) {
          builder.writeU8(item);
        }
      }
      ;
      builder.writeU32(val.timeout_ms);
      builder.writeU64(BigInt(val.request_id));
      break;
    case "Ping":
      builder.writeU8(5);
      break;
  }
}
function decodeClientMessageFields(view) {
  const tag = view.readU8();
  switch (tag) {
    case 0:
      return { type: "Auth", token: view.readString() };
    case 1:
      return { type: "Subscribe", subject: view.readString(), id: view.readU64() };
    case 2:
      return { type: "Unsubscribe", id: view.readU64() };
    case 3:
      return { type: "Publish", subject: view.readString(), payload: (() => {
        const len = view.readU32();
        const arr = [];
        for (let i = 0; i < len; i++) {
          arr.push(view.readU8());
        }
        return arr;
      })() };
    case 4:
      return { type: "Request", subject: view.readString(), payload: (() => {
        const len = view.readU32();
        const arr = [];
        for (let i = 0; i < len; i++) {
          arr.push(view.readU8());
        }
        return arr;
      })(), timeout_ms: view.readU32(), request_id: view.readU64() };
    case 5:
      return { type: "Ping" };
    default:
      throw new Error(`Unknown ClientMessage tag: ${tag}`);
  }
}
function encodeServerMessageFields(val, builder) {
  switch (val.type) {
    case "AuthOk":
      builder.writeU8(0);
      builder.writeString(val.session_id);
      break;
    case "AuthError":
      builder.writeU8(1);
      builder.writeString(val.reason);
      break;
    case "SubscribeOk":
      builder.writeU8(2);
      builder.writeU64(BigInt(val.id));
      break;
    case "SubscribeError":
      builder.writeU8(3);
      builder.writeU64(BigInt(val.id));
      builder.writeString(val.reason);
      break;
    case "Message":
      builder.writeU8(4);
      builder.writeU64(BigInt(val.subscription_id));
      builder.writeString(val.subject);
      {
        builder.writeU32(val.payload.length);
        for (const item of val.payload) {
          builder.writeU8(item);
        }
      }
      ;
      break;
    case "Response":
      builder.writeU8(5);
      builder.writeU64(BigInt(val.request_id));
      {
        builder.writeU32(val.payload.length);
        for (const item of val.payload) {
          builder.writeU8(item);
        }
      }
      ;
      break;
    case "RequestError":
      builder.writeU8(6);
      builder.writeU64(BigInt(val.request_id));
      builder.writeString(val.reason);
      break;
    case "Error":
      builder.writeU8(7);
      builder.writeU32(val.code);
      builder.writeString(val.message);
      break;
    case "Pong":
      builder.writeU8(8);
      break;
  }
}
function decodeServerMessageFields(view) {
  const tag = view.readU8();
  switch (tag) {
    case 0:
      return { type: "AuthOk", session_id: view.readString() };
    case 1:
      return { type: "AuthError", reason: view.readString() };
    case 2:
      return { type: "SubscribeOk", id: view.readU64() };
    case 3:
      return { type: "SubscribeError", id: view.readU64(), reason: view.readString() };
    case 4:
      return { type: "Message", subscription_id: view.readU64(), subject: view.readString(), payload: (() => {
        const len = view.readU32();
        const arr = [];
        for (let i = 0; i < len; i++) {
          arr.push(view.readU8());
        }
        return arr;
      })() };
    case 5:
      return { type: "Response", request_id: view.readU64(), payload: (() => {
        const len = view.readU32();
        const arr = [];
        for (let i = 0; i < len; i++) {
          arr.push(view.readU8());
        }
        return arr;
      })() };
    case 6:
      return { type: "RequestError", request_id: view.readU64(), reason: view.readString() };
    case 7:
      return { type: "Error", code: view.readU32(), message: view.readString() };
    case 8:
      return { type: "Pong" };
    default:
      throw new Error(`Unknown ServerMessage tag: ${tag}`);
  }
}
function encodeInnerDataFields(msg, builder) {
  {
    builder.writeU32(msg.id.length);
    for (const item of msg.id) {
      builder.writeU32(item);
    }
  }
  ;
  {
    builder.writeU32(msg.name.length);
    for (const item of msg.name) {
      builder.writeString(item);
    }
  }
  ;
}
function encodeInnerData(msg) {
  const builder = new PacketBuilder();
  encodeInnerDataFields(msg, builder);
  return builder.build();
}
function decodeInnerDataFields(view) {
  return {
    id: (() => {
      const len = view.readU32();
      const arr = [];
      for (let i = 0; i < len; i++) {
        arr.push(view.readU32());
      }
      return arr;
    })(),
    name: (() => {
      const len = view.readU32();
      const arr = [];
      for (let i = 0; i < len; i++) {
        arr.push(view.readString());
      }
      return arr;
    })()
  };
}
function decodeInnerData(data) {
  const view = new PacketView(data);
  view.skip(1);
  return decodeInnerDataFields(view);
}
function encodeTestDataFields(msg, builder) {
  builder.writeU32(msg.id);
  builder.writeString(msg.name);
  encodeInnerDataFields(msg.inner_data, builder);
}
function encodeTestData(msg) {
  const builder = new PacketBuilder();
  encodeTestDataFields(msg, builder);
  return builder.build();
}
function decodeTestDataFields(view) {
  return {
    id: view.readU32(),
    name: view.readString(),
    inner_data: decodeInnerDataFields(view)
  };
}
function decodeTestData(data) {
  const view = new PacketView(data);
  view.skip(1);
  return decodeTestDataFields(view);
}
function encodeClientEnvelopeFields(msg, builder) {
  encodeClientMessageFields(msg.message, builder);
}
function encodeClientEnvelope(msg) {
  const builder = new PacketBuilder();
  encodeClientEnvelopeFields(msg, builder);
  return builder.build();
}
function decodeClientEnvelopeFields(view) {
  return {
    message: decodeClientMessageFields(view)
  };
}
function decodeClientEnvelope(data) {
  const view = new PacketView(data);
  view.skip(1);
  return decodeClientEnvelopeFields(view);
}
function encodeServerEnvelopeFields(msg, builder) {
  encodeServerMessageFields(msg.message, builder);
}
function encodeServerEnvelope(msg) {
  const builder = new PacketBuilder();
  encodeServerEnvelopeFields(msg, builder);
  return builder.build();
}
function decodeServerEnvelopeFields(view) {
  return {
    message: decodeServerMessageFields(view)
  };
}
function decodeServerEnvelope(data) {
  const view = new PacketView(data);
  view.skip(1);
  return decodeServerEnvelopeFields(view);
}

// src/runtime.ts
var PROTOCOL_VERSION = 109;
var ConnectionState = /* @__PURE__ */ ((ConnectionState2) => {
  ConnectionState2[ConnectionState2["Disconnected"] = 0] = "Disconnected";
  ConnectionState2[ConnectionState2["Connecting"] = 1] = "Connecting";
  ConnectionState2[ConnectionState2["Connected"] = 2] = "Connected";
  ConnectionState2[ConnectionState2["Reconnecting"] = 3] = "Reconnecting";
  ConnectionState2[ConnectionState2["Error"] = 4] = "Error";
  return ConnectionState2;
})(ConnectionState || {});
var DEFAULT_RETRY_CONFIG = {
  maxRetries: 5,
  initialDelayMs: 100,
  maxDelayMs: 3e4,
  backoffMultiplier: 2
};
function calculateRetryDelay(attempt, config = DEFAULT_RETRY_CONFIG) {
  const delay = config.initialDelayMs * Math.pow(config.backoffMultiplier, attempt);
  return Math.min(delay, config.maxDelayMs);
}
async function decompressZstd(data) {
  throw new Error("Zstd decompression not implemented. Import @aspect/zstd or similar.");
}
async function compressZstd(data, level = 3) {
  throw new Error("Zstd compression not implemented. Import @aspect/zstd or similar.");
}
var MottoTransport = class {
  constructor(url, retryConfig = DEFAULT_RETRY_CONFIG) {
    this.url = url;
    this.retryConfig = retryConfig;
  }
  transport = null;
  state = 0 /* Disconnected */;
  retryAttempt = 0;
  retryConfig;
  async connect() {
    if (this.state === 2 /* Connected */) return;
    this.state = 1 /* Connecting */;
    try {
      this.transport = new WebTransport(this.url);
      await this.transport.ready;
      this.state = 2 /* Connected */;
      this.retryAttempt = 0;
    } catch (error) {
      this.state = 4 /* Error */;
      throw error;
    }
  }
  async reconnect() {
    if (this.retryAttempt >= this.retryConfig.maxRetries) {
      throw new Error("Max retry attempts exceeded");
    }
    this.state = 3 /* Reconnecting */;
    const delay = calculateRetryDelay(this.retryAttempt, this.retryConfig);
    this.retryAttempt++;
    await new Promise((resolve) => setTimeout(resolve, delay));
    await this.connect();
  }
  async sendDatagram(data) {
    if (!this.transport || this.state !== 2 /* Connected */) {
      throw new Error("Not connected");
    }
    const writer = this.transport.datagrams.writable.getWriter();
    await writer.write(data);
    writer.releaseLock();
  }
  async *receiveDatagram() {
    if (!this.transport || this.state !== 2 /* Connected */) {
      throw new Error("Not connected");
    }
    const reader = this.transport.datagrams.readable.getReader();
    try {
      while (true) {
        const { value, done } = await reader.read();
        if (done) break;
        yield value;
      }
    } finally {
      reader.releaseLock();
    }
  }
  getState() {
    return this.state;
  }
  async close() {
    if (this.transport) {
      this.transport.close();
      this.transport = null;
    }
    this.state = 0 /* Disconnected */;
  }
};
export {
  ConnectionState,
  DEFAULT_RETRY_CONFIG,
  MottoTransport,
  PROTOCOL_VERSION,
  PROTOCOL_VERSION_BYTE,
  PacketBuilder,
  PacketView,
  SCHEMA_FINGERPRINT,
  SchemaRouterType,
  calculateRetryDelay,
  compressZstd,
  decodeClientEnvelope,
  decodeInnerData,
  decodeServerEnvelope,
  decodeTestData,
  decompressZstd,
  encodeClientEnvelope,
  encodeInnerData,
  encodeServerEnvelope,
  encodeTestData
};
