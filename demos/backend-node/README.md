# Backend Node/TS demo

Scenario: inject into a **bundled Node server artifact** (`.js` / `.cjs` / `.mjs`).

- Injection type: **text/JS artifact** → `revguard strip` restores bit-exact.
- Same production-only gates as frontend when wired through `@revguard/vite` /
  `@revguard/rolldown` / `@revguard/bun` / CLI post-build.
