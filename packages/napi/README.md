# @revguard/napi

Native Node bindings via [napi-rs](https://napi.rs). Built and used in-process
by `@revguard/node-bridge`.

```bash
cd packages/napi
npm i
# Needs a C/C++ toolchain. On this box we used zig as the linker:
#   export CC="zig cc" CXX="zig c++"
npm run build
node -e "console.log(Object.keys(require('./')))"  # inject strip verify
```

API: `inject(dir, configJson?)`, `strip(dir)`, `verify(dir)` — all return JSON strings.
