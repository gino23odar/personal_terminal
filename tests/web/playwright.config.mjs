import { defineConfig } from "@playwright/test";
import { URL } from "node:url";
import process from "node:process";
export default defineConfig({
  testDir: ".",
  testMatch: "terminal.spec.mjs",
  timeout: 20000,
  use: {
    baseURL: "http://127.0.0.1:4180",
    headless: true,
    launchOptions: {
      executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH,
    },
  },
  webServer: {
    command:
      "pnpm --filter @gino-os/web preview --host 127.0.0.1 --port 4180 --strictPort",
    cwd: new URL("../..", import.meta.url).pathname,
    url: "http://127.0.0.1:4180",
    reuseExistingServer: false,
  },
});
