import { URL } from "node:url";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
const routes = new Map([
  ["/parity-runner.mjs", ["../parity/runner.mjs", "text/javascript"]],
  [
    "/pkg/gino_wasm.js",
    ["../../crates/gino-wasm/pkg/gino_wasm.js", "text/javascript"],
  ],
  [
    "/pkg/gino_wasm_bg.wasm",
    ["../../crates/gino-wasm/pkg/gino_wasm_bg.wasm", "application/wasm"],
  ],
  ["/fixtures.mjs", ["./fixtures.mjs", "text/javascript"]],
]);
createServer(async (req, res) => {
  if (req.url === "/") {
    res.writeHead(200, { "Content-Type": "text/html" });
    res.end("<!doctype html><title>WASM bridge test</title>");
    return;
  }
  const route = routes.get(req.url);
  if (!route) {
    res.writeHead(404);
    res.end();
    return;
  }
  try {
    const content = await readFile(new URL(route[0], import.meta.url));
    res.writeHead(200, { "Content-Type": route[1] });
    res.end(content);
  } catch {
    res.writeHead(500);
    res.end();
  }
}).listen(4179, "127.0.0.1");
