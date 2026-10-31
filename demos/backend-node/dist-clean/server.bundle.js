"use strict";
const http = require("http");
const PORT = Number(process.env.PORT || 0);
function handle(req, res) {
  if (req.url === "/health") {
    res.writeHead(200, { "content-type": "application/json" });
    res.end(JSON.stringify({ ok: true, service: "revguard-backend-demo" }));
    return;
  }
  res.writeHead(404);
  res.end("not found");
}
function selfCheck() {
  // Pure logic check used by the inject/strip self-test (must exit).
  const fakeReq = { url: "/health" };
  let status = 0, body = "";
  const fakeRes = {
    writeHead(code) { status = code; },
    end(b) { body = String(b); },
  };
  handle(fakeReq, fakeRes);
  const parsed = JSON.parse(body);
  const ok = status === 200 && parsed.ok === true;
  return { ok, status, body: parsed };
}
function mainListen() {
  const server = http.createServer(handle);
  server.listen(PORT, "127.0.0.1", () => {
    const addr = server.address();
    console.log(JSON.stringify({ listening: true, port: addr.port }));
  });
  return server;
}
if (require.main === module) {
  if (process.argv.includes("--listen")) {
    mainListen();
  } else {
    console.log(JSON.stringify(selfCheck()));
  }
}
module.exports = { handle, selfCheck, mainListen };
