# Testdata

Fixtures used by `scripts/selftest.sh`. These are not copy-paste templates.

| Path | Used by |
| :--- | :--- |
| `sourcemap/` | `extra` — `.map` inject / strip |
| `wasm-section/` | `extra` — custom section inject / strip |
| `swc/` / `swc-wasm/` | `extra` — `@revguard/swc` + WASM plugin |
| `rust-src/` | `extra` — `inject-rust-src` / `strip-rust-src` |
| `python/` | `extra` — PyO3 smoke |

User-facing examples live in `demos/`.
