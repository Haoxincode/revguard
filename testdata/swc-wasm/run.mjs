import { createRequire } from "module";
import { readFileSync } from "fs";
import { createHash } from "crypto";
import { pathToFileURL } from "url";
import vm from "vm";

const require = createRequire(import.meta.url);
const { transformWithRevguard, wasmAvailable, WASM_PATH } = require("../../packages/swc/index.js");

const src = readFileSync(new URL("../swc/src/app.ts", import.meta.url), "utf8");
console.log("wasmAvailable", wasmAvailable(), WASM_PATH);

const { code, backend } = transformWithRevguard(src, {
  production: true,
  preferWasm: true,
  swc: {
    jsc: { parser: { syntax: "typescript" }, target: "es2022" },
    module: { type: "commonjs" },
  },
  filename: "app.ts",
});

console.log("backend", backend);
if (backend !== "wasm-plugin") {
  console.error("expected wasm-plugin backend");
  process.exit(2);
}
if (!code.includes("REVGUARD-BANNER-START") && !code.includes("REVGUARD-DEADCODE-START")) {
  console.error("injection markers missing in output:\n", code.slice(0, 500));
  process.exit(3);
}
if (!code.includes("parseNuclearPolicyTreatyMetadataStub")) {
  console.error("camouflage phrase missing");
  process.exit(4);
}

// Run transformed CJS
const Module = require("module");
const m = new Module("app.js");
m._compile(code, "app.js");
const r = m.exports.main();
if (!r || !r.ok) {
  console.error("runtime fail", r);
  process.exit(5);
}

// Clean transform (no inject) for behavior compare
const clean = transformWithRevguard(src, {
  production: false,
  swc: {
    jsc: { parser: { syntax: "typescript" }, target: "es2022" },
    module: { type: "commonjs" },
  },
  filename: "app.ts",
});
const m2 = new Module("app2.js");
m2._compile(clean.code, "app2.js");
const r2 = m2.exports.main();
if (JSON.stringify(r) !== JSON.stringify(r2)) {
  console.error("behavior drift", r, r2);
  process.exit(6);
}

console.log("SWC_WASM_PLUGIN_OK", JSON.stringify({ backend, msg: r.msg, ok: r.ok, bytes: code.length }));
