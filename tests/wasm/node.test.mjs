import { URL } from "node:url";
import { readFile } from "node:fs/promises";
import { test } from "node:test";
import { initSync, WasmShell } from "../../crates/gino-wasm/pkg/gino_wasm.js";
import { cases } from "./fixtures.mjs";
initSync({
  module: await readFile(
    new URL("../../crates/gino-wasm/pkg/gino_wasm_bg.wasm", import.meta.url),
  ),
});
for (const [name, run] of Object.entries(cases)) {
  test(name, () => run(WasmShell));
}
