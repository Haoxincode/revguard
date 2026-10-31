# Frontend (Vite / Rolldown / Bun) demo

Scenario: inject into a **bundled browser/JS artifact** after a production build.

- Injection type: **text/JS artifact** → `revguard strip` restores bit-exact.
- `dist-clean/app.bundle.js` is the fixture used by `scripts/selftest.sh`.
- `vite.config.example.js` and `bun-build.example.js` show plugin / post-bundle hooks.
