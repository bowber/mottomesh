# MottoMesh Template

> A new way to communicate in my app (and yours).

## Goals
- Use most efficient data formats.
- Use most efficient protocols.
- Use most scalable technologies.
- Browser front-end support (other front-end platforms will be considered later)
- Rust Back-end (and binding to other languages later if needed)

## Key Features
- HTTP-like communication using `Request-Reply` mechanism of NATS
- Bi-directional communication using `Pub-Sub` mechanism of NATS
- Websocket (and NATS protocol for the server) with ease of scaling (by using NATS Cluster and Super Cluster)
- Client-side load-balancing, Geo-Routing
- Type-safe, compact, fast and easy Serialization/Deserialization with the combination of `bitcode-rs` and `wasm-bindgen`

## How to use
### Prerequisites
- [nats-server](https://docs.nats.io/running-a-nats-service/introduction/installation) installed
- Start a local NATS server: `nats-server -c nats.conf`
- [Rust](https://rustup.rs/) installed
- [wasm-pack](https://github.com/drager/wasm-pack) installed

### Export npm package
- Generate the WASM npm package by running `wasm-pack build`
- Publish built npm package to npm with `wasm-pack publish`
- In case you don't want to publish, you can pack the built npm package with `wasm-pack pack` and publish it anywhere else (like a private registry).
- See an example project in `example_web`, run it and open 2 browser tabs to see the communication in action.

### Build server
- Build the Rust server by running `cargo build --release`
- The binary file will be located in `target/release/`
- Or use `cargo run` for development

### Tests
- Test npm package on browser with `wasm-pack test --chrome`
- Test Rust server with `cargo test` (there's no test for the server yet)

## How was this template created? (So you can migrate existing projects)
- Add `bitcode`, `wasm-bindgen`, `async-nats` as dependencies
- Define TestData struct
- Derive TestData struct with traits: `bitcode::Decode`, `bitcode::Encode` and `wasm-bindgen`
- Implement TestData struct with `#[wasm_bindgen]`
- Write tests in `tests/web_tests.rs`


## Key dependencies
- [bitcode-rs](https://docs.rs/bitcode/latest/bitcode/): For serialization/deserialization
- [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen): For binding in the browser
- [NATS.io](https://nats.io): For messaging and communication between services.
- [flate2](https://github.com/flate2-rs/flate2-rs): For compression and decompression
### Not implemented yet
- [UniFFI](https://github.com/mozilla/uniffi-rs): For Android and iOS bindings (do it later for more front-end platforms)
- [napi-rs](https://github.com/napi-rs/napi-rs): For Node.js bindings
