# Electron demo

Electron main + preload (+ renderer) are all JS artifacts — same text injection
as frontend/backend. Wire `@revguard/vite` / Rolldown / Bun for the renderer and
run `revguard inject` on the packaged `main`/`preload` bundles after build.

Injection type: **text/JS artifact** → bit-exact `strip`.

This demo runs under plain Node (`--self-check`) so CI does not need Electron.
