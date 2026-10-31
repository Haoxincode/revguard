import { revguardAfterBundle } from "@revguard/bun";

const result = await Bun.build({
  entrypoints: ["./src/app.js"],
  outdir: "./dist",
  minify: true,
});
if (!result.success) {
  console.error(result.logs);
  process.exit(1);
}
// Injects only when NODE_ENV=production (or pass { force: true } for tests).
await revguardAfterBundle("./dist");
