# Motto Mesh

> A new way to communicate in my app (and yours).

### Goals
- Use most efficient data formats.
- Use most efficient protocols.
- Use most scalable technologies.
- Browser front-end support (other front-end platforms will be considered later)
- Rust Back-end (and binding to other languages later if needed)

### Get Started
- Clone this repo
- 

### Step by step
- Add `bitcode`, `wasm-bindgen`, `async-nats` to your dependencies
- Define for types
- Derive traits: `bitcode::Decode`, `bitcode::Encode`
- Define routes
- Generate js binding

### Key Features
- HTTP-like communication using `Request-Reply` mechanism of NATS
- Realtime communication using `Pub-Sub` mechanism of NATS
- Websocket (and NATS protocol) with ease of scaling (by using NATS Cluster and Super Cluster)
- Client-side load-balancing, Geo-Routing
- Type-safe, compact, fast and easy Serialization/Deserialization with the combination of `bitcode-rs` and `wasm-bindgen`

### Key dependencies
- [bitcode-rs](https://docs.rs/bitcode/latest/bitcode/): For serialization/deserialization
- [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen): For binding in the browser
- [NATS.io](https://nats.io): For messaging and communication between services.
- [UniFFI](https://github.com/mozilla/uniffi-rs): For Android and iOS bindings
