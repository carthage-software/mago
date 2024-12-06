# Fennec WASM

`fennec-wasm` is a WebAssembly (WASM) crate designed to provide high-level functionality for the Fennec toolchain, specifically tailored for use in browser environments.
This crate is primarily intended for the Fennec Playground and similar projects where running Fennec directly in the browser is required.

If you are building applications outside of a browser context, you should use the dedicated Fennec crates instead of this crate.

## Building

To build the WASM module, run the following command:

```sh
wasm-pack build --target web
```

This will generate a `pkg` directory containing the WASM module and associated JavaScript bindings.
