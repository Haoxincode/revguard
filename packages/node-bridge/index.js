/**
 * @revguard/node-bridge
 *
 * Prefer in-process @revguard/napi. Fall back to spawning the `revguard` CLI
 * when the native module is unavailable (wrong platform / not built).
 */
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);

let napi = null;
let napiError = null;
try {
  const candidates = [
    join(__dirname, "../napi/index.js"),
    "@revguard/napi",
  ];
  for (const c of candidates) {
    try {
      if ((c.startsWith("/") || c.startsWith(".")) && !existsSync(c)) continue;
      const mod = require(c);
      if (mod && typeof mod.inject === "function") {
        napi = mod;
        break;
      }
    } catch (e) {
      napiError = e;
    }
  }
} catch (e) {
  napiError = e;
}

export function nativeAvailable() {
  return !!(napi && typeof napi.inject === "function");
}

export function nativeInfo() {
  return {
    available: nativeAvailable(),
    error: napiError ? String(napiError.message || napiError) : null,
  };
}

function resolveBin() {
  if (process.env.REVGUARD_BIN) return process.env.REVGUARD_BIN;
  const candidates = [
    resolve(__dirname, "../../target/release/revguard"),
    resolve(__dirname, "../../target/debug/revguard"),
    "revguard",
  ];
  for (const c of candidates) {
    if (c === "revguard" || existsSync(c)) return c;
  }
  return "revguard";
}

function runCli(args, opts = {}) {
  const bin = resolveBin();
  const r = spawnSync(bin, args, {
    encoding: "utf8",
    env: { ...process.env, ...(opts.env || {}) },
  });
  if (r.error) throw r.error;
  if (r.status !== 0) {
    const err = new Error(
      `revguard ${args.join(" ")} failed (exit ${r.status}):\n${r.stderr || r.stdout}`
    );
    err.status = r.status;
    err.stdout = r.stdout;
    err.stderr = r.stderr;
    throw err;
  }
  return { stdout: r.stdout, stderr: r.stderr, status: r.status, backend: "cli" };
}

/**
 * @param {string} dir
 * @param {{ config?: string, configJson?: string, env?: object, forceCli?: boolean }} [opts]
 */
export function inject(dir, opts = {}) {
  const abs = resolve(dir);
  if (!opts.forceCli && nativeAvailable()) {
    let cfgJson = opts.configJson ?? null;
    if (!cfgJson && opts.config) {
      cfgJson = readFileSync(resolve(opts.config), "utf8");
    }
    const out = napi.inject(abs, cfgJson);
    return { stdout: out, stderr: "", status: 0, backend: "napi" };
  }
  const args = ["inject", abs];
  if (opts.config) args.push("--config", resolve(opts.config));
  return runCli(args, { env: opts.env });
}

export function strip(dir, opts = {}) {
  const abs = resolve(dir);
  if (!opts.forceCli && nativeAvailable()) {
    const out = napi.strip(abs);
    return { stdout: out, stderr: "", status: 0, backend: "napi" };
  }
  return runCli(["strip", abs]);
}

export function verify(dir, opts = {}) {
  const abs = resolve(dir);
  if (!opts.forceCli && nativeAvailable()) {
    const out = napi.verify(abs);
    return { stdout: out, stderr: "", status: 0, backend: "napi" };
  }
  return runCli(["verify", abs]);
}

export function isDisabled() {
  const v = process.env.REVGUARD_DISABLE;
  return v === "1" || (v && v.toLowerCase() === "true");
}

export { resolveBin };
