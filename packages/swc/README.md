# @revguard/swc

Production-only RevGuard injection after `@swc/core` transform.

```js
const { transformWithRevguard } = require("@revguard/swc");
const { code } = transformWithRevguard("export const x = 1;", {
  production: true, // or NODE_ENV=production
});
```

Prefers the WASM visitor in `packages/swc-plugin` when the `.wasm` is present;
otherwise post-emits via `@revguard/napi`. See `packages/swc-plugin/README.md`.
