import { test, expect } from "@playwright/test";
import { readFile } from "node:fs/promises";
import { URL } from "node:url";
const fixtures = JSON.parse(
  await readFile(new URL("../parity/fixtures.json", import.meta.url), "utf8"),
);
for (const fixture of fixtures.filter((fixture) => fixture.smoke)) {
  test(`shared fixture through production browser: ${fixture.name}`, async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByRole("status")).toHaveText("Ready");
    let transcript =
      "GINO.OS — browser virtual shell\nType 'help' for available commands.\n";
    for (const step of fixture.steps) {
      const input = page.getByRole("textbox", { name: "Command" });
      await input.fill(step.input);
      await input.press("Enter");
      await expect(input).toHaveValue("");
      await expect(
        page.getByText(`Exit status: ${step.expected.status}`, { exact: true }),
      ).toBeVisible();
      transcript += `visitor@gino.os:/home/visitor$ ${step.input}\n`;
      for (const event of step.expected.events) {
        if (event.type === "clearScreen") transcript = "";
        else transcript += event.text;
      }
      // Exact text including newlines: no whitespace normalization hides differences.
      await expect
        .poll(() => page.getByRole("log").textContent())
        .toBe(transcript);
      await expect(page.getByRole("status")).toHaveText(
        step.expected.session.exitStatus === null ? "Ready" : "Session closed",
      );
    }
  });
}
