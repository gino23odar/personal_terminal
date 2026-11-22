import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { URL } from "node:url";
import { expectedTranscript } from "./runner.mjs";
const fixtures = JSON.parse(
  await readFile(new URL("./fixtures.json", import.meta.url), "utf8"),
);
const native = JSON.parse(
  await readFile(new URL("./generated/native.json", import.meta.url), "utf8"),
);
for (const [index, fixture] of fixtures.entries()) {
  test(`native / browser WASM parity: ${fixture.name}`, async ({ page }) => {
    await page.goto("http://127.0.0.1:4179/");
    const actual = await page.evaluate(async (fixture) => {
      const module = await import("/pkg/gino_wasm.js");
      await module.default();
      const { runFixtures } = await import("/parity-runner.mjs");
      return runFixtures(module.WasmShell, [fixture]);
    }, fixture);
    expect(actual).toEqual([native[index]]);
    expect(actual).toEqual(expectedTranscript([fixture]));
  });
}
