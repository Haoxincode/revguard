/**
 * @revguard/swc
 *
 * Two production injection paths:
 * 1. **WASM plugin** (preferred when .wasm is built): `jsc.experimental.plugins`
 * 2. **Post-emit injectString** via @revguard/napi (fallback)
 */
const swc = require("@swc/core");
const { createRequire } = require("module");
const { existsSync } = require("fs");
const { join } = require("path");

const requireFromHere = createRequire(__filename);

const WASM_PATH = join(
  __dirname,
  "../swc-plugin/revguard_swc_plugin.wasm"
);

function loadNapi() {
  const candidates = [
    join(__dirname, "../napi/index.js"),
    "@revguard/napi",
  ];
  for (const c of candidates) {
    try {
      if ((c.startsWith("/") || c.startsWith(".")) && !existsSync(c)) continue;
      const m = requireFromHere(c);
      if (m && typeof m.injectString === "function") return m;
    } catch (_) {}
  }
  return null;
}

function isProd(opts = {}) {
  if (opts.force) return true;
  if (process.env.REVGUARD_DISABLE === "1") return false;
  return process.env.NODE_ENV === "production" || opts.production === true;
}

function wasmAvailable() {
  return existsSync(WASM_PATH);
}

/**
 * @param {object} [opts]
 * @param {boolean} [opts.preferWasm=true] use WASM plugin when available
 * @param {boolean} [opts.forcePostEmit=false] skip WASM, use napi injectString
 */
function transformWithRevguard(code, opts = {}) {
  const preferWasm = opts.preferWasm !== false && !opts.forcePostEmit;
  const useWasm = preferWasm && wasmAvailable() && isProd(opts);

  const swcOpts = {
    jsc: {
      parser: { syntax: "ecmascript", jsx: true },
      target: "es2022",
      ...(opts.swc && opts.swc.jsc ? opts.swc.jsc : {}),
      experimental: {
        ...((opts.swc && opts.swc.jsc && opts.swc.jsc.experimental) || {}),
        ...(useWasm
          ? { plugins: [[WASM_PATH, opts.pluginConfig || {}]] }
          : {}),
      },
    },
    module: { type: "es6", ...((opts.swc && opts.swc.module) || {}) },
    filename: opts.filename || "input.js",
  };

  // Merge top-level swc opts carefully without clobbering jsc we built
  if (opts.swc) {
    for (const k of Object.keys(opts.swc)) {
      if (k !== "jsc" && k !== "module") swcOpts[k] = opts.swc[k];
    }
  }

  const result = swc.transformSync(code, swcOpts);
  let out = result.code;
  let backend = useWasm ? "wasm-plugin" : "none";

  if (isProd(opts) && !useWasm) {
    const napi = loadNapi();
    if (!napi) throw new Error("neither WASM plugin nor @revguard/napi available");
    let cfg = null;
    if (typeof opts.revguardConfig === "string") cfg = opts.revguardConfig;
    else if (opts.revguardConfig) cfg = JSON.stringify(opts.revguardConfig);
    out = napi.injectString(out, cfg);
    backend = "napi-postemit";
  }

  return { code: out, map: result.map, backend, wasmPath: useWasm ? WASM_PATH : null };
}

module.exports = {
  transformWithRevguard,
  wasmAvailable,
  WASM_PATH,
  isProd,
};
