# @revguard/node-bridge

Node bindings for RevGuard.

1. **Prefer** in-process `@revguard/napi` (napi-rs native module).
2. **Fallback** to spawning the `revguard` CLI (`REVGUARD_BIN` or in-repo binary).

```js
import { inject, strip, verify, nativeInfo } from "@revguard/node-bridge";
console.log(nativeInfo()); // { available: true, error: null }
inject("./dist");
```

Force CLI: `inject(dir, { forceCli: true })`.
