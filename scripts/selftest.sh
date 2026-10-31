#!/usr/bin/env bash
# Unified RevGuard self-test.
# Usage: ./scripts/selftest.sh [core|napi|extra|all]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export REVGUARD_ROOT="$ROOT"

RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
pass(){ echo "${GRN}PASS${RST} $*"; }
fail(){ echo "${RED}FAIL${RST} $*"; exit 1; }
info(){ echo "${YLW}INFO${RST} $*"; }
sha(){ sha256sum "$1" | awk '{print $1}'; }

resolve_bin() {
  if [[ -n "${REVGUARD_BIN:-}" && -x "$REVGUARD_BIN" ]]; then
    printf '%s\n' "$REVGUARD_BIN"
    return
  fi
  local target
  target="$(cd "$ROOT" && cargo metadata --format-version 1 --no-deps \
    | python3 -c "import json,sys; print(json.load(sys.stdin)['target_directory'])")"
  if [[ ! -x "$target/release/revguard" ]]; then
    info "building release binary…"
    (cd "$ROOT" && cargo build --release)
    target="$(cd "$ROOT" && cargo metadata --format-version 1 --no-deps \
      | python3 -c "import json,sys; print(json.load(sys.stdin)['target_directory'])")"
  fi
  [[ -x "$target/release/revguard" ]] || fail "revguard binary not found at $target/release/revguard"
  printf '%s\n' "$target/release/revguard"
}

BIN="$(resolve_bin)"
export REVGUARD_BIN="$BIN"

# ---------- core: JS inject → strip hash path ----------
suite_core() {
  local WORK=/tmp/rg-selftest-core
  local CLEAN_SRC="$ROOT/demos/frontend/dist-clean/app.bundle.js"

  rm -rf "$WORK"
  mkdir -p "$WORK"
  cp "$CLEAN_SRC" "$WORK/app.bundle.js"

  local CLEAN_HASH INJ_HASH OUT_CLEAN OUT_INJ STRIP_HASH STRIP2 DIS_HASH PROBE PROBE_OUT
  CLEAN_HASH="$(sha "$WORK/app.bundle.js")"
  info "clean hash = $CLEAN_HASH"

  info "1) run clean bundle"
  OUT_CLEAN="$(node "$WORK/app.bundle.js")"
  echo "    output: $OUT_CLEAN"
  echo "$OUT_CLEAN" | grep -q '"ok":true' || fail "clean bundle did not report ok"

  info "2) revguard inject"
  "$BIN" inject "$WORK" --config "$ROOT/revguard.config.json"
  [[ -f "$WORK/revguard.manifest.json" ]] || fail "manifest not written"
  INJ_HASH="$(sha "$WORK/app.bundle.js")"
  [[ "$INJ_HASH" != "$CLEAN_HASH" ]] || fail "injected hash unexpectedly equal to clean"
  pass "injected (hash changed $CLEAN_HASH → $INJ_HASH)"

  info "3) run injected bundle (must match clean output)"
  OUT_INJ="$(node "$WORK/app.bundle.js")"
  echo "    output: $OUT_INJ"
  [[ "$OUT_INJ" == "$OUT_CLEAN" ]] || fail "runtime output changed after inject"
  pass "runtime output identical"

  info "4) static proof that dead-code never executes"
  grep -q 'REVGUARD-DEADCODE-START' "$WORK/app.bundle.js" || fail "dead-code marker missing"
  grep -q '__REVGUARD_NEVER__' "$WORK/app.bundle.js" || fail "dead-code guard missing"
  PROBE="$WORK/probe_deadcode.mjs"
  cat > "$PROBE" <<'PROBE'
import { readFileSync } from "node:fs";
import vm from "node:vm";

const src = readFileSync(process.argv[2], "utf8");
const bag = { out: null, exit: null };
const ctx = vm.createContext({
  console: { log: (...a) => { bag.out = a.join(" "); } },
  process: { exit: (c) => { bag.exit = c; } },
  globalThis: {},
  Symbol,
});
vm.runInContext(src, ctx, { filename: "app.bundle.js" });
const guard = vm.runInContext(
  'typeof globalThis !== "undefined" && globalThis.__REVGUARD_NEVER__ === Symbol.for("revguard.never")',
  ctx
);
if (guard !== false) {
  console.error("dead-code guard unexpectedly truthy:", guard);
  process.exit(2);
}
const names = [...src.matchAll(/function\s+([A-Za-z_][A-Za-z0-9_]{20,})\s*\(/g)]
  .map(m => m[1])
  .filter(n => /Stub|Placeholder|Noop|Unused|Only/i.test(n));
let leaked = 0;
for (const n of names) {
  if (Object.prototype.hasOwnProperty.call(ctx.globalThis, n)) leaked++;
}
console.log(JSON.stringify({ guardFalse: guard === false, camouflageFns: names.length, leaked }));
if (leaked !== 0) process.exit(3);
PROBE
  PROBE_OUT="$(node "$PROBE" "$WORK/app.bundle.js")"
  echo "    probe: $PROBE_OUT"
  echo "$PROBE_OUT" | grep -q '"guardFalse":true' || fail "dead-code guard not proven false"
  echo "$PROBE_OUT" | grep -q '"leaked":0' || fail "dead-code identifiers leaked to globalThis"
  pass "dead-code guard is false; zero camouflage fns leaked"

  info "5) revguard verify"
  "$BIN" verify "$WORK"
  pass "verify ok"

  info "6) revguard strip → hash must equal clean"
  "$BIN" strip "$WORK"
  STRIP_HASH="$(sha "$WORK/app.bundle.js")"
  echo "    stripped hash = $STRIP_HASH"
  [[ "$STRIP_HASH" == "$CLEAN_HASH" ]] || fail "stripped hash $STRIP_HASH != clean $CLEAN_HASH"
  pass "stripped hash == clean hash"

  "$BIN" strip "$WORK"
  STRIP2="$(sha "$WORK/app.bundle.js")"
  [[ "$STRIP2" == "$CLEAN_HASH" ]] || fail "second strip changed hash"
  pass "strip is idempotent"

  info "7) REVGUARD_DISABLE=1 must no-op"
  cp "$CLEAN_SRC" "$WORK/app.bundle.js"
  REVGUARD_DISABLE=1 "$BIN" inject "$WORK" --config "$ROOT/revguard.config.json"
  DIS_HASH="$(sha "$WORK/app.bundle.js")"
  [[ "$DIS_HASH" == "$CLEAN_HASH" ]] || fail "REVGUARD_DISABLE did not prevent inject"
  if [[ -f "$WORK/revguard.manifest.json" ]]; then
    grep -q '"files": \[\]' "$WORK/revguard.manifest.json" \
      || fail "manifest should have empty files when disabled"
  fi
  pass "REVGUARD_DISABLE=1 honored"
}

# ---------- napi: in-process smoke + scenario demos ----------
js_scenario() {
  local name="$1"
  local src_dir="$2"
  shift 2
  local files=("$@")
  info "scenario [$name] text/JS artifact inject→run→strip"
  local work="/tmp/rg-scenario-$name"
  rm -rf "$work"
  mkdir -p "$work"
  local -a clean_hashes=()
  local -a baseline=()
  local f out h i
  for f in "${files[@]}"; do
    cp "$src_dir/$f" "$work/$f"
    clean_hashes+=("$(sha "$work/$f")")
  done
  for f in "${files[@]}"; do
    baseline+=("$(node "$work/$f")")
  done

  node --input-type=module <<JS
import { inject, verify, strip } from "$ROOT/packages/node-bridge/index.js";
const r = inject("$work", { config: "$ROOT/revguard.config.json" });
if (r.backend !== "napi") { console.error("backend", r.backend); process.exit(2); }
const v = JSON.parse(verify("$work").stdout);
if (v.ok !== v.checked || v.checked < 1) { console.error(v); process.exit(3); }
console.log("injected via", r.backend, "files", v.checked);
JS

  i=0
  for f in "${files[@]}"; do
    out="$(node "$work/$f")"
    [[ "$out" == "${baseline[$i]}" ]] || fail "[$name] $f output changed: $out vs ${baseline[$i]}"
    i=$((i+1))
  done

  node --input-type=module <<JS
import { strip } from "$ROOT/packages/node-bridge/index.js";
strip("$work");
JS

  i=0
  for f in "${files[@]}"; do
    h="$(sha "$work/$f")"
    [[ "$h" == "${clean_hashes[$i]}" ]] || fail "[$name] $f hash $h != ${clean_hashes[$i]}"
    i=$((i+1))
  done
  pass "[$name] behavior unchanged + strip hash restore"
}

suite_napi() {
  info "A) napi native module smoke (in-process via node-bridge)"
  node --input-type=module <<JS
import {
  inject, strip, verify, nativeInfo, nativeAvailable,
} from "$ROOT/packages/node-bridge/index.js";
import { cpSync, rmSync, mkdirSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";

const info = nativeInfo();
console.log("nativeInfo:", JSON.stringify(info));
if (!nativeAvailable()) {
  console.error("napi not available:", info.error);
  process.exit(2);
}

const work = "/tmp/rg-napi-smoke";
rmSync(work, { recursive: true, force: true });
mkdirSync(work, { recursive: true });
cpSync("$ROOT/demos/frontend/dist-clean/app.bundle.js", work + "/app.bundle.js");

const clean = readFileSync(work + "/app.bundle.js");
const cleanHash = createHash("sha256").update(clean).digest("hex");

let r = inject(work, { config: "$ROOT/revguard.config.json" });
if (r.backend !== "napi") {
  console.error("expected napi backend, got", r.backend);
  process.exit(3);
}
console.log("inject backend=napi");
r = verify(work);
if (r.backend !== "napi") process.exit(4);
const v = JSON.parse(r.stdout);
if (v.ok !== 1 || v.checked !== 1) {
  console.error("verify failed", v);
  process.exit(5);
}
console.log("verify ok", r.stdout.trim());

const { spawnSync } = await import("node:child_process");
const run = spawnSync("node", [work + "/app.bundle.js"], { encoding: "utf8" });
if (run.status !== 0 || !run.stdout.includes('"ok":true')) {
  console.error("runtime broken", run.stdout, run.stderr);
  process.exit(6);
}

r = strip(work);
if (r.backend !== "napi") process.exit(7);
const after = readFileSync(work + "/app.bundle.js");
const afterHash = createHash("sha256").update(after).digest("hex");
if (afterHash !== cleanHash) {
  console.error("hash mismatch", { cleanHash, afterHash });
  process.exit(8);
}
console.log("strip hash match", afterHash);
console.log("NAPI_SMOKE_OK");
JS
  pass "napi in-process inject/verify/strip"

  js_scenario "frontend" "$ROOT/demos/frontend/dist-clean" "app.bundle.js"
  js_scenario "backend-node" "$ROOT/demos/backend-node/dist-clean" "server.bundle.js"

  info "backend-node live HTTP check after inject"
  local WORK_BE=/tmp/rg-scenario-backend-live
  rm -rf "$WORK_BE" && mkdir -p "$WORK_BE"
  cp "$ROOT/demos/backend-node/dist-clean/server.bundle.js" "$WORK_BE/"
  node --input-type=module <<JS
import { inject } from "$ROOT/packages/node-bridge/index.js";
inject("$WORK_BE", { config: "$ROOT/revguard.config.json" });
JS
  node "$WORK_BE/server.bundle.js" --listen > "$WORK_BE/out.txt" 2>&1 &
  local BE_PID=$!
  sleep 0.3
  local PORT HEALTH
  PORT="$(node -e "const fs=require('fs'); const t=fs.readFileSync('$WORK_BE/out.txt','utf8'); const m=t.match(/\"port\":(\\d+)/); if(!m) process.exit(1); process.stdout.write(m[1]);")"
  HEALTH="$(curl -fsS "http://127.0.0.1:${PORT}/health")"
  kill "$BE_PID" 2>/dev/null || true
  wait "$BE_PID" 2>/dev/null || true
  echo "    health=$HEALTH"
  echo "$HEALTH" | grep -q '"ok":true' || fail "backend health check failed"
  pass "backend-node live HTTP after inject"

  js_scenario "wasm-js-glue" "$ROOT/demos/wasm-js-glue/dist-clean" "app_glue.js"
  js_scenario "electron" "$ROOT/demos/electron-app/dist-clean" "main.bundle.js" "preload.bundle.js"

  info "scenario [native-rust] compile-time markers"
  unset CARGO_TARGET_DIR || true
  local NATIVE="$ROOT/demos/native-rust"
  (
    cd "$NATIVE"
    info "  build WITH markers"
    cargo clean -q
    cargo build --release 2>&1 | tail -5
  )
  local BIN_MARKED="$NATIVE/target/release/revguard-native-demo"
  local OUT_MARKED
  OUT_MARKED="$($BIN_MARKED)"
  echo "    output: $OUT_MARKED"
  echo "$OUT_MARKED" | grep -q '"ok":true' || fail "marked binary bad output"
  local STRINGS_HIT
  STRINGS_HIT="$(python3 "$ROOT/scripts/count_needles.py" "$BIN_MARKED")"
  echo "    strings hits=$STRINGS_HIT / 5"
  [[ "$STRINGS_HIT" -ge 4 ]] || fail "expected camouflage strings in binary"
  pass "native-rust marked binary runs + strings visible"

  info "  build CLEAN (no markers) = restore path"
  (cd "$NATIVE" && cargo build --release --no-default-features 2>&1 | tail -5)
  local BIN_CLEAN="$NATIVE/target/release/revguard-native-demo"
  local OUT_CLEAN
  OUT_CLEAN="$($BIN_CLEAN)"
  echo "    output: $OUT_CLEAN"
  [[ "$OUT_CLEAN" == "$OUT_MARKED" ]] || fail "clean vs marked output differ (behavior must match)"
  if strings "$BIN_CLEAN" | grep -q 'REVGUARD-BANNER-START'; then
    fail "clean binary still contains REVGUARD banner"
  fi
  pass "native-rust clean rebuild has same behavior, no banner strings"
}

# ---------- extra: SWC, PyO3, sourcemap, rotation, rust-src, wasm ----------
suite_extra() {
  local PY="$ROOT/.venv/bin/python"
  export PATH="/tmp/toolchains/zig:$ROOT/.venv/bin:$PATH"
  export CC="${CC:-zig cc}" CXX="${CXX:-zig c++}"

  info "1) SWC transform + injectString"
  node <<'JS'
const ROOT = process.env.REVGUARD_ROOT;
const { transformWithRevguard } = require(ROOT + "/packages/swc");
const fs = require("fs");
const src = fs.readFileSync(ROOT + "/testdata/swc/src/app.ts", "utf8");
const { code, backend } = transformWithRevguard(src, {
  production: true,
  forcePostEmit: true,
  swc: { jsc: { parser: { syntax: "typescript" }, target: "es2022" }, module: { type: "commonjs" } },
});
if (backend !== "napi-postemit") { console.error("expected napi-postemit", backend); process.exit(1); }
if (!code.includes("REVGUARD-BANNER-START")) { console.error("no banner"); process.exit(2); }
if (!code.includes("REVGUARD-DEADCODE-START")) { console.error("no deadcode"); process.exit(3); }
const Module = require("module");
const m = new Module("app.js");
m._compile(code, "app.js");
const exp = m.exports;
const r = exp.main ? exp.main() : exp;
if (!r || !r.ok) { console.error("runtime", r); process.exit(4); }
const { stripString } = require(ROOT + "/packages/napi");
const stripped = stripString(code);
if (stripped.includes("REVGUARD-BANNER")) { console.error("strip failed"); process.exit(5); }
const { transformWithRevguard: t2 } = require(ROOT + "/packages/swc");
const clean = t2(src, {
  production: false,
  swc: { jsc: { parser: { syntax: "typescript" }, target: "es2022" }, module: { type: "commonjs" } },
}).code;
const m2 = new Module("app2.js");
m2._compile(clean, "app2.js");
const r2 = m2.exports.main();
if (JSON.stringify(r) !== JSON.stringify(r2)) { console.error("behavior drift", r, r2); process.exit(6); }
console.log("SWC_OK", JSON.stringify(r));
JS
  pass "SWC post-process inject + behavior + stripString"

  info "1b) SWC WASM plugin via jsc.experimental.plugins"
  local WASM="$ROOT/packages/swc-plugin/revguard_swc_plugin.wasm"
  [[ -f "$WASM" ]] || fail "missing $WASM — rebuild with: (cd packages/swc-plugin && cargo build --target wasm32-wasip1 --release && cp target/wasm32-wasip1/release/revguard_swc_plugin.wasm .)"
  node "$ROOT/testdata/swc-wasm/run.mjs"
  node <<'JS'
const ROOT = process.env.REVGUARD_ROOT;
const { transformWithRevguard } = require(ROOT + "/packages/swc");
const { stripString } = require(ROOT + "/packages/napi");
const fs = require("fs");
const src = fs.readFileSync(ROOT + "/testdata/swc/src/app.ts", "utf8");
const { code, backend } = transformWithRevguard(src, {
  production: true,
  preferWasm: true,
  swc: { jsc: { parser: { syntax: "typescript" }, target: "es2022" }, module: { type: "commonjs" } },
  filename: "app.ts",
});
if (backend !== "wasm-plugin") { console.error("backend", backend); process.exit(2); }
if (!code.includes("REVGUARD-BANNER-START") || !code.includes("REVGUARD-DEADCODE-START")) {
  console.error("markers missing"); process.exit(3);
}
const stripped = stripString(code);
if (stripped.includes("REVGUARD-BANNER") || stripped.includes("REVGUARD-DEADCODE")) {
  console.error("strip failed on wasm output"); process.exit(4);
}
console.log("SWC_WASM_STRIP_OK");
JS
  pass "SWC WASM plugin inject + behavior + strip"

  info "2) PyO3 bindings"
  "$PY" "$ROOT/testdata/python/smoke.py"
  pass "PyO3 inject/verify/strip hash"

  info "3) Source map inject/strip"
  local WORK=/tmp/rg-sourcemap
  rm -rf "$WORK" && mkdir -p "$WORK"
  cp "$ROOT/testdata/sourcemap/app.js.map" "$WORK/"
  cat > "$WORK/cfg.json" <<'JSON'
{
  "enabled": true,
  "inject_banner": true,
  "inject_dead_code": true,
  "inject_source_map": true,
  "inject_wasm_section": false,
  "intensity": "medium",
  "categories": ["cbrn", "toxin", "explosive", "weapon"]
}
JSON
  "$BIN" inject "$WORK" --config "$WORK/cfg.json"
  "$BIN" strip "$WORK" || true
  rm -f "$WORK/revguard.manifest.json"
  local CLEAN_MAP_HASH
  CLEAN_MAP_HASH="$(sha "$WORK/app.js.map")"
  info "  canonical clean map hash=$CLEAN_MAP_HASH"
  "$BIN" inject "$WORK" --config "$WORK/cfg.json"
  grep -q 'REVGUARD-BANNER-START' "$WORK/app.js.map" || fail "banner missing in map"
  grep -q 'x_revguard_phrases' "$WORK/app.js.map" || fail "x_revguard_phrases missing"
  "$PY" -c "import json; json.load(open('$WORK/app.js.map')); print('json-ok')"
  "$BIN" verify "$WORK"
  "$BIN" strip "$WORK"
  local STRIP_HASH
  STRIP_HASH="$(sha "$WORK/app.js.map")"
  [[ "$STRIP_HASH" == "$CLEAN_MAP_HASH" ]] || fail "sourcemap hash $STRIP_HASH != $CLEAN_MAP_HASH"
  "$PY" - <<PY
import json
m=json.load(open("$WORK/app.js.map"))
assert m["version"]==3 and "sourcesContent" in m
assert "x_revguard_phrases" not in m
assert "REVGUARD" not in m["sourcesContent"][0]
print("structure-ok")
PY
  pass "sourcemap inject/strip hash + JSON structure"

  info "4) Wordlist rotation"
  WORK=/tmp/rg-rotate
  rm -rf "$WORK" && mkdir -p "$WORK"
  cp "$ROOT/demos/frontend/dist-clean/app.bundle.js" "$WORK/"
  cat > "$WORK/cfg.json" <<JSON
{
  "enabled": true,
  "inject_banner": true,
  "inject_dead_code": true,
  "intensity": "high",
  "categories": ["cbrn", "toxin", "explosive", "weapon"],
  "wordlist_path": "$ROOT/wordlists/rotated.example.json",
  "wordlist_version": "rot2026-example"
}
JSON
  "$BIN" inject "$WORK" --config "$WORK/cfg.json"
  grep -q 'rot2026_' "$WORK/app.bundle.js" || fail "rotated phrases not present"
  grep -q 'wordlist_id' "$WORK/revguard.manifest.json"
  grep -q 'rot2026-example' "$WORK/revguard.manifest.json" || fail "wordlist version not in manifest"
  node "$WORK/app.bundle.js" | grep -q '"ok":true' || fail "runtime after rotate"
  "$BIN" strip "$WORK"
  pass "wordlist rotation override + strip"

  info "5) inject-rust-src / strip-rust-src"
  unset CARGO_TARGET_DIR || true
  local RDEMO=/tmp/rg-rust-src
  rm -rf "$RDEMO"
  cp -a "$ROOT/testdata/rust-src" "$RDEMO"
  local HASH_SRC_BEFORE
  HASH_SRC_BEFORE="$(sha "$RDEMO/src/main.rs")"
  "$BIN" inject-rust-src "$RDEMO"
  grep -q 'REVGUARD-RUST-SRC-START' "$RDEMO/src/main.rs" || fail "rust marker missing"
  grep -q 'revguard_markers' "$RDEMO/Cargo.toml" || fail "feature missing"
  (cd "$RDEMO" && cargo build --release 2>&1 | tail -3)
  local OUT_MARKED
  OUT_MARKED="$("$RDEMO/target/release/revguard-rust-src-demo")"
  echo "  marked output: $OUT_MARKED"
  echo "$OUT_MARKED" | grep -q '"ok":true' || fail "marked bin bad"
  "$PY" - <<'PY'
from pathlib import Path
b=Path("/tmp/rg-rust-src/target/release/revguard-rust-src-demo").read_bytes()
assert b"REVGUARD-BANNER-START" in b
print("binary-has-banner")
PY
  "$BIN" strip-rust-src "$RDEMO"
  grep -q 'REVGUARD-RUST-SRC-START' "$RDEMO/src/main.rs" && fail "marker still present after strip"
  [[ "$(sha "$RDEMO/src/main.rs")" == "$HASH_SRC_BEFORE" ]] || fail "main.rs hash not restored"
  grep -q 'revguard_markers' "$RDEMO/Cargo.toml" && fail "feature still in Cargo.toml"
  (cd "$RDEMO" && cargo build --release --offline 2>&1 | tail -3)
  local OUT_CLEAN
  OUT_CLEAN="$("$RDEMO/target/release/revguard-rust-src-demo")"
  [[ "$OUT_CLEAN" == "$OUT_MARKED" ]] || fail "behavior changed clean vs marked"
  pass "inject-rust-src + strip-rust-src + binary strings + behavior"

  info "6) WASM custom section"
  WORK=/tmp/rg-wasm-section
  rm -rf "$WORK" && mkdir -p "$WORK"
  cp "$ROOT/testdata/wasm-section/module.wasm" "$WORK/"
  local CLEAN_W INJ_W
  CLEAN_W="$(sha "$WORK/module.wasm")"
  cat > "$WORK/cfg.json" <<'JSON'
{
  "enabled": true,
  "inject_wasm_section": true,
  "inject_banner": false,
  "inject_dead_code": false,
  "inject_source_map": false,
  "intensity": "medium",
  "categories": ["cbrn", "toxin", "explosive", "weapon"]
}
JSON
  "$BIN" inject "$WORK" --config "$WORK/cfg.json"
  INJ_W="$(sha "$WORK/module.wasm")"
  [[ "$INJ_W" != "$CLEAN_W" ]] || fail "wasm hash unchanged"
  "$PY" - <<PY
from pathlib import Path
b=Path("$WORK/module.wasm").read_bytes()
assert b[:4]==b'\\x00asm'
assert b'RevGuard' in b or b'revguard' in b or b'REVGUARD' in b
assert b'parseNuclear' in b or b'lookupToxin' in b or b'Stub' in b
print('wasm-section-payload-ok', len(b))
PY
  "$BIN" verify "$WORK"
  "$BIN" strip "$WORK"
  [[ "$(sha "$WORK/module.wasm")" == "$CLEAN_W" ]] || fail "wasm strip hash mismatch"
  pass "wasm custom section inject/strip hash"
}

SUITE="${1:-core}"
case "$SUITE" in
  core)
    suite_core
    echo
    echo "${GRN}===== CORE SELF-TESTS PASSED =====${RST}"
    ;;
  napi)
    suite_napi
    echo
    echo "${GRN}===== NAPI + SCENARIO SELF-TESTS PASSED =====${RST}"
    ;;
  extra)
    suite_extra
    echo
    echo "${GRN}===== EXTRA SELF-TESTS PASSED =====${RST}"
    ;;
  all)
    suite_core
    suite_napi
    suite_extra
    echo
    echo "${GRN}===== ALL SELF-TESTS PASSED =====${RST}"
    ;;
  *)
    echo "Usage: $0 [core|napi|extra|all]" >&2
    exit 2
    ;;
esac
