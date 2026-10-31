/**
 * @revguard/bun
 *
 * Bun has no formal plugin hook for post-bundle mutation equivalent to
 * Vite/Rolldown's closeBundle, so this package exposes:
 *   1. `revguardAfterBundle(outDir, opts)` — call after `Bun.build`.
 *   2. A thin `bunfig`-friendly helper script entry.
 *
 * Injects only when NODE_ENV=production (or opts.force). Honors REVGUARD_DISABLE=1.
 */
import { inject, isDisabled, strip, verify } from "@revguard/node-bridge";
import { resolve } from "node:path";

/**
 * @param {string} outDir
 * @param {object} [opts]
 * @param {boolean} [opts.enabled=true]
 * @param {string}  [opts.config]
 * @param {boolean} [opts.force=false]
 */
export async function revguardAfterBundle(outDir, opts = {}) {
  const enabled = opts.enabled !== false;
  const prod =
    opts.force === true || process.env.NODE_ENV === "production";
  if (!enabled || isDisabled() || !prod) {
    return { skipped: true };
  }
  const dir = resolve(outDir);
  const r = inject(dir, { config: opts.config });
  return { skipped: false, stdout: r.stdout };
}

export { strip as revguardStrip, verify as revguardVerify, isDisabled };

export default revguardAfterBundle;
