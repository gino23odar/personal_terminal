import { test } from "@playwright/test";
import { cases } from "./fixtures.mjs";
for (const name of Object.keys(cases)) {
  test(name, async ({ page }) => {
    await page.goto("http://127.0.0.1:4179/");
    await page.evaluate(async (caseName) => {
      const module = await import("/pkg/gino_wasm.js");
      await module.default();
      const fixtures = await import("/fixtures.mjs");
      await fixtures.cases[caseName](module.WasmShell);
    }, name);
  });
}
