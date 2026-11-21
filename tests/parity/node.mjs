import console from "node:console";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFile, mkdir, writeFile, rm } from "node:fs/promises";
import { fileURLToPath, URL } from "node:url";
import { initSync, WasmShell } from "../../crates/gino-wasm/pkg/gino_wasm.js";
import { runFixtures, expectedTranscript } from "./runner.mjs";
const root = fileURLToPath(new URL("../..", import.meta.url));
const generated = new URL("./generated/", import.meta.url);
// Never leave a stale native baseline when the current run fails.
await rm(generated, { recursive: true, force: true });
const fixtures = JSON.parse(
  await readFile(new URL("./fixtures.json", import.meta.url), "utf8"),
);
const native = JSON.parse(
  execFileSync(
    "cargo",
    ["run", "--quiet", "--locked", "-p", "gino-shell", "--example", "parity"],
    {
      cwd: root,
      encoding: "utf8",
      timeout: 120000,
      maxBuffer: 4 * 1024 * 1024,
    },
  ),
);
assert.deepStrictEqual(
  native,
  expectedTranscript(fixtures),
  "native vs reviewed expectations",
);
initSync({
  module: await readFile(
    new URL("../../crates/gino-wasm/pkg/gino_wasm_bg.wasm", import.meta.url),
  ),
});
assert.deepStrictEqual(
  await runFixtures(WasmShell, fixtures),
  native,
  "Node WASM vs native",
);
await mkdir(generated, { recursive: true });
await writeFile(new URL("native.json", generated), JSON.stringify(native));
console.log(
  `Parity passed: ${fixtures.length} scenarios, ${native.reduce((n, s) => n + s.steps.length, 0)} steps; native Rust = Node WASM = reviewed expectations.`,
);
