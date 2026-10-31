"use strict";
// Simulated wasm-bindgen / wasm-pack JS glue. The .wasm binary is NOT touched.
function stubWasmMemory() {
  return { buffer: new ArrayBuffer(8) };
}
async function init(wasmPath) {
  // In a real app this would WebAssembly.instantiateStreaming(...).
  // Here we only prove the glue keeps working after RevGuard inject.
  void wasmPath;
  const memory = stubWasmMemory();
  return {
    memory,
    add(a, b) { return (a + b) | 0; },
    greet(name) { return `hello from glue, ${name}`; },
  };
}
async function main() {
  const api = await init("./app.wasm");
  const out = { sum: api.add(20, 22), msg: api.greet("wasm"), ok: api.add(20, 22) === 42 };
  console.log(JSON.stringify(out));
  return out;
}
if (typeof require !== "undefined" && require.main === module) {
  main().then((r) => { if (!r.ok) process.exit(1); });
}
module.exports = { init, main };
