# rs-stellar-rpc

Rust version of a stellar smart contract rpc server.

Run with `cargo run`. Example frontend is at: https://github.com/paulbellamy/stellar-wasm-demo-next

## TODO

- [ ] inject chain state
  - [ ] fetch latest archive
  - [ ] stream txmetas since then, rebuilding the current chain state
  - [ ] pass the chain state into the wasm runtime
- [ ] implement host functions
