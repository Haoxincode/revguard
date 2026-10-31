# revguard_swc_plugin — true SWC WASM visitor plugin

Status: **built and working** on current stable rustc (**1.99.1** verified).

## What it does

A `wasm32-wasip1` SWC transform plugin (`VisitMut`) that, in production
transforms loaded via `@swc/core` `jsc.experimental.plugins`, injects:

1. A `/*! … */` banner comment (`REVGUARD-BANNER-START` … `END`) with
   camouflage CBRN/toxin/explosive/weapon **identifier** strings
2. A dead-code IIFE gated by
   `globalThis.__REVGUARD_NEVER__ === Symbol.for("revguard.never")`
   (`REVGUARD-DEADCODE-START` … `END`)

No recipes, dosages, or procedures — identifiers only. Runtime behavior of
the host module is unchanged.

## Toolchain (verified)

| Piece | Version |
|-------|---------|
| rustc | **stable / 1.99+** |
| target | `wasm32-wasip1` |
| `swc_core` | **83.0.0** with feature `ecma_plugin_transform` (do **not** depend on `swc_common` directly) |
| resolved `swc_common` | **27.0.0** (via Cargo.lock) |
| serde | **1.0.229** (lockfile) |
| host `@swc/core` | ≥ 1.15 (demo uses 1.16.x) |

Artifact: `revguard_swc_plugin.wasm` at this package root (and under
`target/wasm32-wasip1/release/` when rebuilt).

## Build

```bash
# one-time
rustup target add wasm32-wasip1   # on your active stable toolchain

cd packages/swc-plugin
unset CARGO_TARGET_DIR
cargo build --target wasm32-wasip1 --release
cp target/wasm32-wasip1/release/revguard_swc_plugin.wasm .
```

`.cargo/config.toml` sets `--cfg=swc_ast_unknown` for wasip1. Keep **Cargo.lock**
committed.

## Load from JS

`@revguard/swc` prefers the WASM plugin when the `.wasm` file exists:

```js
const { transformWithRevguard, wasmAvailable } = require("@revguard/swc");
const { code, backend } = transformWithRevguard(src, {
  production: true,
  preferWasm: true, // default when .wasm present
  swc: { jsc: { parser: { syntax: "typescript" }, target: "es2022" } },
});
// backend === "wasm-plugin"
```

Under the hood: `jsc.experimental.plugins: [[absolutePathToWasm, {}]]`.

Fallback: if the `.wasm` is missing, `@revguard/swc` uses `@revguard/napi`
`injectString` post-emit (`backend === "napi-postemit"`).

## Self-test

```bash
node testdata/swc-wasm/run.mjs
# → SWC_WASM_PLUGIN_OK {"backend":"wasm-plugin",…}

./scripts/selftest.sh extra   # includes the WASM plugin step
```
