import { renderToStaticMarkup } from "react-dom/server";
import { expect, it } from "vitest";
import { App } from "./App";
it("shows loading state and prevents execution before the runtime is ready", () => {
  const html = renderToStaticMarkup(<App />);
  expect(html).toContain("Loading shell");
  expect(html).toContain("readOnly");
  expect(html).toContain("disabled");
  expect(html).toContain('aria-label="Command"');
});
