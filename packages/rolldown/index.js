/**
 * @revguard/rolldown
 *
 * Rolldown plugin. Injects only when NODE_ENV=production (or opts.force).
 * Honors REVGUARD_DISABLE=1.
 */
import { inject, isDisabled } from "@revguard/node-bridge";
import { join } from "node:path";

/**
 * @param {object} [opts]
 * @param {boolean} [opts.enabled=true]
 * @param {string}  [opts.config]
 * @param {string}  [opts.outDir="dist"]
 * @param {boolean} [opts.force=false]  inject even outside production (tests)
 */
export function revguardRolldown(opts = {}) {
  const enabled = opts.enabled !== false;
  const outDir = opts.outDir || "dist";

  return {
    name: "revguard",
    closeBundle() {
      const prod =
        opts.force === true ||
        process.env.NODE_ENV === "production" ||
        process.env.ROLLDOWN_MODE === "production";
      if (!enabled || isDisabled() || !prod) return;
      const dir = join(process.cwd(), outDir);
      const r = inject(dir, { config: opts.config });
      // eslint-disable-next-line no-console
      console.log(`[revguard] injected into ${dir}`);
      if (r.stdout) console.log(r.stdout.trimEnd());
    },
  };
}

export default revguardRolldown;
