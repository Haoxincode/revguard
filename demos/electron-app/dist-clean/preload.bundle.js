"use strict";
function bridgeApi() {
  return { ping: () => "pong", source: "preload" };
}
function selfCheck() {
  const api = bridgeApi();
  return { ok: api.ping() === "pong", api };
}
if (require.main === module) {
  console.log(JSON.stringify(selfCheck()));
}
module.exports = { bridgeApi, selfCheck };
