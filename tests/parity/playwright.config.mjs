import { defineConfig } from "@playwright/test";
import bridge from "../wasm/playwright.config.mjs";
export default defineConfig({
  ...bridge,
  testDir: ".",
  testMatch: "browser.spec.mjs",
});
