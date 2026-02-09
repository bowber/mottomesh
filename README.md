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
- **Motto-generated protocol + payload SDKs** from a single Rust schema
- **Automatic reconnection** with subscription restoration

## Project Structure

```
mottomesh/
├── schema/                 # Motto source schema + generated SDKs
│   ├── src/schema.rs       # Single source of truth for protocol + payload types
│   └── generated/          # Generated Rust + TypeScript SDKs
├── crates/
│   ├── gateway/            # Gateway server (WebTransport + WebSocket)
│   └── server/             # Backend service example
├── client-ts/              # TypeScript client library
├── example_web/            # Example SolidJS web app
└── nats.conf               # NATS server configuration
```

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)
- [nats-server](https://docs.nats.io/running-a-nats-service/introduction/installation)
- [motto](https://crates.io/crates/motto) CLI (`cargo install motto --version 0.3.2`)
- Node.js 18+

### 1. Start NATS Server

```bash
nats-server -c nats.conf
```

### 2. Generate SDKs from Motto Schema

```bash
# one-command helper (defaults to patch bump)
./scripts/regen-sdk.sh

# optional: choose bump level
./scripts/regen-sdk.sh minor
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
import { encodeTestData, decodeTestData } from '@motto/schema';

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
  const data = decodeTestData(msg.payload);
  console.log('Received:', data.name);
});

// Publish a message
const data = {
  id: 1,
  name: 'hello',
  inner_data: {
    id: [1, 1, 1],
    name: ['hello', 'hello', 'hello'],
  },
};
await client.publish('messages', encodeTestData(data));

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

### Build Rust Workspace

```bash
cargo build
```

### Run Tests

```bash
# Run all Rust tests
cargo test

# Run TypeScript client tests
cd client-ts
pnpm exec vitest run
```

### Build TypeScript Client

```bash
cd client-ts
pnpm build
```

### Type Checking

```bash
# TypeScript type checking
cd client-ts
pnpm lint
```

## Key Dependencies

- **[wtransport](https://github.com/BiagioFesta/wtransport)**: WebTransport implementation
- **[axum](https://github.com/tokio-rs/axum)**: WebSocket server framework
- **[async-nats](https://github.com/nats-io/nats.rs)**: NATS client
- **[motto](https://crates.io/crates/motto)**: Schema-first multi-platform SDK generation

## Future Enhancements

- [ ] Connection rate limiting (sub-crate)
- [ ] Metrics and observability
- [ ] TLS certificate auto-renewal (ACME)
- [ ] UniFFI bindings for mobile
- [ ] napi-rs bindings for Node.js
