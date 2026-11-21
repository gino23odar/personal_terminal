/* global DataTransfer, ClipboardEvent, document, window, KeyboardEvent */
import { test, expect } from "@playwright/test";
async function ready(page) {
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("Ready");
}
async function run(page, command) {
  const input = page.getByRole("textbox", { name: "Command" });
  await input.fill(command);
  await input.press("Enter");
  await expect(input).toHaveValue("");
  await expect(page.getByRole("status")).not.toHaveText("Running…");
}
test("production browser executes Rust commands and recovers from errors", async ({
  page,
}) => {
  await ready(page);
  // Once initialized, commands must work with all network requests blocked.
  await page.route("**/*", (route) => route.abort());
  const log = page.getByRole("log");
  for (const [command, output] of [
    ["help", "whoami"],
    ['echo "hello 🦀"', "hello 🦀"],
    ["pwd", "/home/visitor"],
    ["whoami", "visitor"],
  ]) {
    await run(page, command);
    await expect(log).toContainText(output);
  }
  await run(page, "missing");
  await expect(page.getByText("Exit status: 127")).toBeVisible();
  await run(page, "echo 'broken");
  await expect(page.getByText("Exit status: 2")).toBeVisible();
  await run(page, "echo recovered");
  await expect(page.getByText("Exit status: 0")).toBeVisible();
  await expect(page.locator(".xterm")).toBeVisible();
  await expect(page.locator(".xterm-rows")).toContainText("recovered");
  await run(page, "clear");
  await expect(log).toHaveText("");
  await expect(page.locator(".xterm-rows")).not.toContainText("recovered");
  await run(page, "exit 7");
  await expect(page.getByRole("status")).toHaveText("Session closed");
  await expect(
    page.getByRole("button", { name: "Run", exact: true }),
  ).toBeDisabled();
  await page.getByRole("button", { name: "New session" }).click();
  await run(page, "echo fresh");
  await expect(log).toContainText("fresh");
});
test("keyboard editing, history, cancellation, EOF and paste protection", async ({
  page,
}) => {
  await ready(page);
  const input = page.getByRole("textbox", { name: "Command" });
  await run(page, "echo first");
  await input.fill("draft");
  await input.press("ArrowUp");
  await expect(input).toHaveValue("echo first");
  await input.press("ArrowDown");
  await expect(input).toHaveValue("draft");
  await input.press("End");
  await input.press("Control+c");
  await expect(input).toHaveValue("");
  await input.fill("echo hellx");
  await input.press("Backspace");
  await input.press("o");
  await input.press("Enter");
  await expect(page.getByRole("log")).toContainText("hello");
  await input.evaluate((element) => {
    const data = new DataTransfer();
    data.setData("text/plain", "echo surprise\nexit");
    element.dispatchEvent(
      new ClipboardEvent("paste", {
        bubbles: true,
        cancelable: true,
        clipboardData: data,
      }),
    );
  });
  await expect(page.getByRole("alert")).toContainText("Paste rejected");
  await expect(input).toHaveValue("");
  await input.press("Control+d");
  await expect(page.getByRole("status")).toHaveText("Session closed");
});
test("sessions are independent and input is not executed from URLs or HTML", async ({
  page,
  context,
}) => {
  await page.goto("/?command=exit");
  await expect(page.getByRole("status")).toHaveText("Ready");
  await run(page, `echo '<img src=x onerror=alert(1)>'`);
  await expect(page.locator(".shell img")).toHaveCount(0);
  await expect(page.getByRole("log")).toContainText(
    "<img src=x onerror=alert(1)>",
  );
  await run(page, "exit");
  const other = await context.newPage();
  await ready(other);
  await run(other, "whoami");
  await expect(other.getByRole("log")).toContainText("visitor");
  await other.close();
});
test("failed WASM load is explicit and retry creates a usable session", async ({
  page,
}) => {
  await page.route("**/*.wasm", (route) => route.abort());
  await page.goto("/");
  await expect(page.getByRole("status")).toHaveText("Shell unavailable");
  await expect(
    page.getByRole("button", { name: "Run", exact: true }),
  ).toBeDisabled();
  await page.unroute("**/*.wasm");
  await page.getByRole("button", { name: "Reload shell" }).click();
  await expect(page.getByRole("status")).toHaveText("Ready");
  await run(page, "echo retry");
  await expect(page.getByRole("log")).toContainText("retry");
});
test("terminal resizes without horizontal page overflow", async ({ page }) => {
  await ready(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await run(page, "echo narrow");
  await expect(page.getByRole("log")).toContainText("narrow");
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.setViewportSize({ width: 1200, height: 800 });
  await run(page, "echo wide");
  await expect(page.getByRole("log")).toContainText("wide");
});
test("same-turn Ctrl+C cancels Rust execution and duplicate submissions are ignored", async ({
  page,
}) => {
  await ready(page);
  const input = page.getByRole("textbox", { name: "Command" });
  await input.fill("exit 9");
  await input.evaluate((element) => {
    element.setSelectionRange(element.value.length, element.value.length);
    element.form.requestSubmit();
    element.form.requestSubmit();
    element.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "c",
        ctrlKey: true,
        bubbles: true,
        cancelable: true,
      }),
    );
  });
  await expect(page.getByText("Exit status: 130")).toBeVisible();
  await expect(page.getByRole("status")).toHaveText("Ready");
  await run(page, "echo after cancellation");
  await expect(page.getByRole("log")).toContainText("after cancellation");
});
