# MottoMesh

> A modern real-time communication framework with WebTransport and WebSocket support.

## Architecture

```
┌─────────────┐   WebTransport/WS   ┌─────────────┐      NATS       ┌────────┐
│   Browser   │ ─────────────────── │   Gateway   │ ──────────────  │ Server │
│  (TS Client)│  https://443        │   (Rust)    │   tcp://4222    │ (Rust) │
└─────────────┘                     └─────────────┘                 └────────┘
      │                                    │
      │ JWT Auth                           │ JWT Validation
      ▼                                    │ Permission Checks
┌─────────────┐                            │ Message Routing
│ Auth Server │ ◄──────────────────────────┘
│  (separate) │
└─────────────┘
```

## Key Features

- **WebTransport** (HTTP/3 over QUIC) with automatic **WebSocket fallback**
- **JWT-based authentication** with flexible permission system
- **NATS-style subject patterns** with wildcard support (`*` and `>`)
- **Bi-directional real-time communication** via pub/sub
- **Request-reply pattern** support
- **Type-safe serialization** with bitcode + WASM bindings
- **Automatic reconnection** with subscription restoration

## Project Structure

```
mottomesh/
├── crates/
│   ├── mottomesh/          # Shared library (data types, WASM bindings)
│   ├── gateway/            # Gateway server (WebTransport + WebSocket)
│   └── server/             # Backend service example
├── client-ts/              # TypeScript client library
├── example_web/            # Example SolidJS web app
├── pkg/                    # Generated WASM package
└── nats.conf               # NATS server configuration
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [nats-server](https://docs.nats.io/running-a-nats-service/introduction/installation)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- Node.js 18+

### 1. Start NATS Server

```bash
nats-server -c nats.conf
```

### 2. Build WASM Package

```bash
cd crates/mottomesh
wasm-pack build --target web
```

### 3. Start the Gateway

```bash
# Set the JWT secret (required)
export JWT_SECRET="your-secret-key"

# Optional: configure ports
export GATEWAY_PORT=4433  # WebTransport port (WebSocket on 4434)
export NATS_URL=localhost:4222

# Run the gateway
cargo run -p mottomesh-gateway
```

### 4. Start the Backend Server

```bash
cargo run -p mottomesh-server
```

### 5. Run the Example Web App

```bash
cd example_web
pnpm install
pnpm dev
```

## TypeScript Client Usage

```typescript
import { MottomeshClient } from '@mottomesh/client';
import { TestData } from 'mottomesh';

// Create client
const client = new MottomeshClient({
  url: 'https://localhost:4433',
  token: 'your-jwt-token',
  transport: 'auto',  // WebTransport with WebSocket fallback
});

// Connect
await client.connect();

// Subscribe to messages
const sub = client.subscribe('messages', (msg) => {
  const data = TestData.decode(msg.payload);
  console.log('Received:', data.name());
});

// Publish a message
const data = new TestData(1, 'hello');
await client.publish('messages', data.encode());

// Clean up
await sub.unsubscribe();
await client.disconnect();
```

## JWT Token Format

The gateway expects JWT tokens with these claims:

```json
{
  "sub": "user-id",
  "exp": 1234567890,
  "iat": 1234567890,
  "permissions": ["publish", "subscribe", "request"],
  "allowed_subjects": ["messages.*", "user.>"],
  "deny_subjects": ["admin.*"]
}
```

### Subject Patterns

- `*` matches a single token: `messages.*` matches `messages.user1` but not `messages.user1.inbox`
- `>` matches one or more tokens: `messages.>` matches `messages.user1` and `messages.user1.inbox`

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `JWT_SECRET` | (required) | Secret key for JWT validation |
| `GATEWAY_HOST` | `0.0.0.0` | Host to bind gateway |
| `GATEWAY_PORT` | `4433` | WebTransport port (WebSocket on port+1) |
| `NATS_URL` | `localhost:4222` | NATS server URL |
| `TLS_CERT_PATH` | (none) | Path to TLS certificate (auto-generates if not set) |
| `TLS_KEY_PATH` | (none) | Path to TLS private key |

## Development

### Build All Crates

```bash
cargo build
```

### Run Tests

```bash
# Run all Rust tests (71 tests)
cargo test

# Run TypeScript client tests (53 tests)
cd client-ts
npm test

# WASM tests (requires Chrome)
cd crates/mottomesh
wasm-pack test --chrome
```

#### Test Coverage

| Component | Tests | Coverage |
|-----------|-------|----------|
| `crates/mottomesh` | 13 | TestData encoding/decoding, edge cases |
| `crates/gateway` | 58 | JWT, permissions, sessions, codec, handler |
| `client-ts` | 53 | Codec, messages, client API |
| **Total** | **124** | |

### Build TypeScript Client

```bash
cd client-ts
npm run build
```

### Type Checking

```bash
# TypeScript type checking
cd client-ts
npm run lint
```

## Key Dependencies

- **[wtransport](https://github.com/BiagioFesta/wtransport)**: WebTransport implementation
- **[axum](https://github.com/tokio-rs/axum)**: WebSocket server framework
- **[async-nats](https://github.com/nats-io/nats.rs)**: NATS client
- **[bitcode](https://docs.rs/bitcode)**: Efficient binary serialization
- **[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)**: WASM bindings

## Future Enhancements

- [ ] Connection rate limiting (sub-crate)
- [ ] Metrics and observability
- [ ] TLS certificate auto-renewal (ACME)
- [ ] UniFFI bindings for mobile
- [ ] napi-rs bindings for Node.js
