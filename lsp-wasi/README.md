# The LSP based on WASI 

This crate sets up the LSP for being compiled to the WASI target.

NOTE: This is currently not more than a PoC, and doesn't even work.

## bindgen

NOTE: This will fail due to a bug in `wasm-bindgen`.

```shell
wasm-bindgen --out-dir ../seedwing-enforcer-vscode-addon/src/ffi ./target/wasm32-wasi/release/seedwing_enforcer_lsp_wasi.wasm
```