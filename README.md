# 🛡️ RevGuard

中文版：[README.zh-CN.md](./README.zh-CN.md)

**A speed bump for AI-assisted reverse engineering.**

Reverse engineering minified code used to take effort. Today, anyone can paste a production bundle into a well-aligned LLM, type *"explain this minified code,"* and get a clean decompilation in seconds.

**RevGuard adds friction to that workflow.** It turns the safety guardrails of commercial LLMs (like ChatGPT, Claude, etc.) against casual reverse engineers. Strictly during production builds, RevGuard injects harmless, academic-sounding identifiers (related to CBRN, toxins, explosives, and weapons) into your artifacts. When someone blindly pastes your bundle into an LLM, it is much more likely to trip safety filters, causing the model to hesitate or outright refuse the request.

* **Zero Runtime Impact**: No encryption, no control-flow obfuscation, no renaming of your existing symbols, and **zero changes to runtime behavior**.
* **Bit-Exact Reversibility**: Comes with a cryptographic manifest. One command strips the markers and restores your build to the **exact byte-for-byte clean state**.
* **Clear Scope**: Think of it as deterrence (a speed bump), not DRM (a vault door).

```text
Clean Source ──► Bundler ──► Production Artifact
                                    │
                     RevGuard Inject (Prod-only)
                                    ▼
                            Artifact + Markers
                         + revguard.manifest.json
                                    │
              ├──► Pasted into LLM  ──► 🚨 Trips safety policy (Refusal)
              └──► revguard strip   ──► ✨ Bit-exact clean artifact (for SAST/Store review)
```

---

## ⚡ 60-Second Quickstart

**1. Build the core CLI**
```bash
cargo build --release
export REVGUARD_BIN=$PWD/target/release/revguard
```

**2. Hook into your bundler (e.g., Vite — injects only during `vite build` in production)**
```js
// vite.config.js
import { defineConfig } from "vite";
import { revguardVite } from "@revguard/vite";

export default defineConfig({
  plugins: [revguardVite()],
});
```

**3. Or run the CLI manually in your CI/CD pipeline**
```bash
./target/release/revguard inject ./dist --config ./revguard.config.json
./target/release/revguard verify ./dist
./target/release/revguard strip  ./dist   # Strip clean before sending to SAST or App Stores!
```
> 💡 **Kill Switch**: Setting `REVGUARD_DISABLE=1` turns all plugins and integrations into a complete no-op.

---

## ⚙️ How It Works

1. **Top Banner**: Prepends a legal `/*! … */` preserved comment block packed with decoy academic identifiers.
2. **Dead-Code Branch**: Appends a trailing branch guarded by `globalThis.__REVGUARD_NEVER__ === Symbol.for("revguard.never")` (always evaluates to `false` at runtime and never executes, yet resists static dead-code elimination by minifiers).
3. **Manifest Tracking**: Emits `revguard.manifest.json` recording the `sha256_before`, `sha256_after`, and injection metadata for every modified file.
4. **Bit-Exact Strip**: Running `revguard strip` restores text/JS artifacts back to their exact original pre-injection SHA-256 hashes.

---

## 🚨 Red Lines & Safety Policy

RevGuard is strictly scoped to **four categories**, each offering `low`, `medium`, and `high` intensity levels:

| Category | Theme (Academic Labels Only) |
| :--- | :--- |
| **`cbrn`** | Chemical, biological, radiological, or nuclear academic labels |
| **`toxin`** | Toxin and lethal-dose (e.g., LD50) related academic labels |
| **`explosive`** | Explosive and IED-related academic labels |
| **`weapon`** | Weapon manufacturing and ballistics academic labels |

* **Harmless Decoys Only**: Every injected string is a dummy identifier, such as `computeLd50LethalDoseStubNoDataUnused`. There are **no recipes, no synthesis steps, no dosages, and no actionable instructions**.
* **Permanent Prohibitions**: Never includes CSAM/minors, pornography, human trafficking, NCII, or any real-world actionable harm. (Enforced in `crates/core/src/wordlist.rs` and `wordlists/README.md`).

---

## 📦 Installation & Ecosystem

### Rust CLI
> ⚠️ Note: The CLI is a native Rust binary named `revguard` (crate path `crates/cli`), not a global npm package.

```bash
cargo install --path crates/cli
# Or build directly from the workspace:
cargo build --release
./target/release/revguard --help
```

### JS / TS Packages (Workspace)
Ensure `export REVGUARD_BIN=$PWD/target/release/revguard` is set if falling back to the CLI.

| Package | Role & Behavior |
| :--- | :--- |
| `@revguard/node-bridge` | Node bindings — prefers in-process `napi`, gracefully falls back to CLI |
| `@revguard/vite` | Vite plugin — `revguardVite()` (Rolldown-compatible) |
| `@revguard/rolldown` | Rolldown plugin — `revguardRolldown()` |
| `@revguard/bun` | Bun `Bun.build` post-bundle hook — `revguardAfterBundle()` |
| `@revguard/napi` | High-performance native module via `napi-rs` |
| `@revguard/swc` | `@swc/core` helper — prefers true WASM plugin, falls back to `napi` |
| `revguard_swc_plugin` | `wasm32-wasip1` SWC AST visitor (`.wasm` shipped alongside package) |

<details>
<summary><b>👉 Click to expand: Rolldown / Bun / SWC / Python Integration Examples</b></summary>

```js
// 1. Rolldown
import { revguardRolldown } from "@revguard/rolldown";
export default { plugins: [revguardRolldown({ outDir: "dist" })] };

// 2. Bun
import { revguardAfterBundle } from "@revguard/bun";
const result = await Bun.build({ entrypoints: ["./src/index.ts"], outdir: "./dist", minify: true });
if (result.success) await revguardAfterBundle("./dist"); // No-op unless NODE_ENV=production

// 3. SWC (@swc/core) — Prefers true WASM visitor when .wasm is present
const { transformWithRevguard } = require("@revguard/swc");
const { code, backend } = transformWithRevguard(src, { production: true });
// backend === "wasm-plugin" | "napi-postemit"
```

```bash
# 4. Python (PyO3)
cd packages/pyo3 && maturin develop --release
python -c "import revguard; print(revguard.inject('./dist')[:80])"
```
</details>

### Production-Only Gating
**Injection is strictly an artifact property, never a source-code property:**

| Integration | Gating Condition |
| :--- | :--- |
| `@revguard/vite` | `command === "build"` AND `production` mode |
| `@revguard/rolldown` / `bun` | `NODE_ENV=production` (or explicit `opts.force`) |
| `@revguard/swc` | `production: true` or `NODE_ENV=production` (or `force`) |
| CLI `revguard inject` | **Always runs** (you control when to invoke it in CI) |
| Env `REVGUARD_DISABLE=1` | **Global override** — turns all integrations into no-ops |

---

## 🛠️ Configuration & CLI Reference

**Configuration (`revguard.config.json`)**
```json
{
  "enabled": true,
  "categories": ["cbrn", "toxin", "explosive", "weapon"],
  "intensity": "medium",
  "inject_banner": true,
  "inject_dead_code": true,
  "inject_source_map": false,
  "inject_wasm_section": false,
  "wordlist_path": null,
  "wordlist_version": null
}
```
*Supported Environment Variables: `REVGUARD_DISABLE`, `REVGUARD_BIN`, `REVGUARD_WORDLIST`, `REVGUARD_INJECT_SOURCEMAP`, `REVGUARD_INJECT_WASM`.*

**CLI Commands**
```bash
revguard inject <dir> [--config PATH]            # Inject markers into build directory
revguard strip <dir>                             # Restore bit-exact clean files via manifest
revguard verify <dir>                            # Verify artifact hashes against manifest
revguard inject-rust-src <crate-dir> [--config]  # Inject compile-time markers into Rust source
revguard strip-rust-src <crate-dir>              # Remove markers from Rust source
revguard export-wordlist [wordlists/builtin.json]# Export built-in wordlist for custom rotation
```

---

## 🌐 Multi-Scenario Matrix

RevGuard operates in two distinct modes:
1. **Text / JS Artifacts**: Modifies bundled `.js/.mjs/.cjs/.ts` files post-build. Fully supports **bit-exact restoration** via `revguard strip`.
2. **Compile-Time Source**: Bakes keepalive string constants into native binaries. Restoring requires **stripping the source markers and recompiling**.

| Target Scenario | Approach | Mode | Demo Reference |
| :--- | :--- | :--- | :--- |
| **Frontend (Vite/Rolldown/Bun)** | Attach bundler plugin to production build | Text/JS | `demos/frontend/` + selftests |
| **Backend (Node/TS)** | Run `inject` or `node-bridge` post-bundle | Text/JS | `demos/backend-node/` |
| **Electron** | Main, preload, and renderer are all JS artifacts | Text/JS | `demos/electron-app/` |
| **Web / WASM** | Injects JS glue (`wasm-bindgen` output) by default; optional `.wasm` custom section | Text/JS (+Binary) | `demos/wasm-js-glue/`<br>`testdata/wasm-section/` |
| **Tauri** | Frontend uses JS plugin; Rust core uses compile-time markers | Hybrid | Plugin + `demos/native-rust/` |
| **Native Binary (Rust)** | `#[used]` + `black_box` string constants (via `inject-rust-src` or manual) | Compile-Time | `demos/native-rust/`<br>`testdata/rust-src/` |
| **Native (Go/C/C++/Zig)** | Equivalent keepalive string constants | Compile-Time | See `demos/native-rust/README.md` |

### Advanced Capabilities
* ✅ **True SWC Plugin**: Native WASM AST visitor (`rustc stable/1.99+`, `swc_core 83`) with `napi` `injectString` fallback.
* ✅ **PyO3 Bindings**: Located in `packages/pyo3` (built with `maturin` — `import revguard`).
* ✅ **Source Map Poisoning**: Setting `inject_source_map: true` injects markers into `.map` `sourcesContent` (fully reversible via `strip`).
* ✅ **Wordlist Rotation**: Customize or rotate wordlists via `wordlist_path` or `REVGUARD_WORDLIST`.
* ⚠️ **WASM Custom Section (PoC)**: Setting `inject_wasm_section: true` appends a custom `revguard` section. *Caveats: `wasm-opt` or certain bundlers may strip unknown sections; LLMs rarely inspect raw binary sections unless dumped first (lower deterrence than JS glue banners); alters whole-module SRI hashes as expected.*

---

## 🗂️ Architecture & Self-Tests

```text
crates/
  core              Core logic: wordlist, inject, strip, sourcemap, wasm, rust_src...
  cli               CLI binary
packages/
  ├── napi          napi-rs native bindings
  ├── node-bridge   Node wrapper (prefers napi, falls back to CLI)
  ├── vite / rolldown / bun  Bundler plugins
  ├── swc / swc-plugin   @swc/core helper & wasm32-wasip1 visitor
  └── pyo3          Python bindings (maturin)
demos/              User-facing scenario templates
testdata/           Self-test fixtures (SWC, sourcemap, rust-src, wasm section, PyO3)
wordlists/          builtin.json, rotated.example.json
```

**Run the Verification Suite:**
```bash
cargo build --release
./scripts/selftest.sh            # core: JS inject → strip bit-exact hash loop
./scripts/selftest.sh napi       # napi + frontend/backend/wasm/electron/native
./scripts/selftest.sh extra      # SWC (WASM), PyO3, sourcemaps, rotation, rust-src, wasm section
./scripts/selftest.sh all
# napi extra requires: npm run build:napi  (g++ or zig c++)
```

---

## ⚠️ Honest Disclaimer & SAST Warning (Read This!)

1. **Deterrence, Not DRM**: RevGuard is a speed bump against casual, copy-paste AI reverse engineering. It is **not** encryption, DRM, or license enforcement, and it does not replace proper IP protection. Determined analysts, uncensored local models, or custom system prompts can bypass it. Use it as one quirky layer in a defense-in-depth strategy.
2. **Friendly Fire (SAST & App Store Reviews)**: Because decoy identifiers mimic high-risk terminology, **they may trigger SAST scanners, malware heuristics, supply-chain audits, and App Store / Google Play review pipelines.**
   * **Best Practice**: **Always run `revguard strip ./dist` before submitting artifacts to security scanners or app stores.** Reserve the injected builds strictly for final end-user distribution—and give your internal security team a heads-up first!

---

## 🗺️ Roadmap
- [ ] Richer WASM `name-section` injection techniques
- [ ] Additional host bundler hooks (Webpack, esbuild, etc.)

## 📄 License
[MIT](./LICENSE)
