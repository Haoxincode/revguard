/**
 * @revguard/vite
 *
 * Vite plugin. Injects ONLY when `command === "build"` AND
 * `mode === "production"` (or NODE_ENV=production). Dev server never injects.
 * Honors REVGUARD_DISABLE=1.
 *
 * Compatible with Vite's Rolldown-backed builds (Vite 6+/7).
 */
import { inject, isDisabled } from "@revguard/node-bridge";
import { join } from "node:path";

/**
 * @param {object} [opts]
 * @param {boolean} [opts.enabled=true]
 * @param {string}  [opts.config] path to revguard config JSON
 * @param {string}  [opts.outDir]  override outDir (default: vite resolved outDir)
 */
export function revguardVite(opts = {}) {
  const enabled = opts.enabled !== false;
  let outDir = opts.outDir || null;
  let shouldInject = false;

  return {
    name: "revguard",
    apply: "build",
    configResolved(config) {
      outDir = outDir || config.build.outDir || "dist";
      // Vite sets command + mode. Only production builds.
      shouldInject =
        enabled &&
        !isDisabled() &&
        config.command === "build" &&
        (config.mode === "production" || process.env.NODE_ENV === "production");
    },
    closeBundle() {
      if (!shouldInject) return;
      const dir = join(process.cwd(), outDir);
      try {
        const r = inject(dir, { config: opts.config });
        this.info?.(`[revguard] injected into ${dir}\n${r.stdout}`);
        // eslint-disable-next-line no-console
        console.log(`[revguard] injected into ${dir}`);
        if (r.stdout) console.log(r.stdout.trimEnd());
      } catch (e) {
        this.error?.(e) || console.error("[revguard] inject failed:", e);
        throw e;
      }
    },
  };
}

export default revguardVite;
