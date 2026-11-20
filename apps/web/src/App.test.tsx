import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("development landing page", () => {
  it("discloses that the shell is unavailable without offering command input", () => {
    const html = renderToStaticMarkup(<App />);
    expect(html).toContain("Shell not connected yet");
    expect(html).toContain("No commands are available.");
    expect(html).not.toMatch(/<(input|textarea|form)\b/);
  });
});
