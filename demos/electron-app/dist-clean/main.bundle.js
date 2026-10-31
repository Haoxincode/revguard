"use strict";
// Minimal Electron main-process stand-in. We do NOT require 'electron' so the
// self-test can run under plain Node: it exposes the same pure helpers the
// real main would use, plus a --self-check mode.
function createWindowConfig() {
  return { width: 800, height: 600, title: "revguard-electron-demo" };
}
function appVersion() {
  return "0.1.0-demo";
}
function selfCheck() {
  const cfg = createWindowConfig();
  const ok = cfg.width === 800 && appVersion().startsWith("0.1.0");
  return { ok, cfg, version: appVersion() };
}
if (require.main === module) {
  console.log(JSON.stringify(selfCheck()));
}
module.exports = { createWindowConfig, appVersion, selfCheck };
