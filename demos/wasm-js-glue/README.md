# WASM JS-glue demo

RevGuard injects the **JS glue** that wasm-bindgen / wasm-pack emits (the
`.js` wrapper), not the `.wasm` binary by default.

### Optional: `.wasm` custom section

You can opt in with `inject_wasm_section: true` (see `testdata/wasm-section/`).
Custom sections may be stripped by `wasm-opt` / some bundlers, change SRI
hashes, and are often invisible to LLMs unless a tool dumps them. Prefer JS
glue for the main deterrent path.
