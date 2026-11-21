import { URL } from "node:url";
import process from "node:process";
import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: ".",
  testMatch: "browser.spec.mjs",
  timeout: 15000,
  use: {
    browserName: "chromium",
    headless: true,
    launchOptions: {
      executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH,
    },
  },
  webServer: {
    command: "node tests/wasm/server.mjs",
    cwd: new URL("../..", import.meta.url).pathname,
    url: "http://127.0.0.1:4179/",
    reuseExistingServer: false,
  },
});
