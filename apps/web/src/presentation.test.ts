import { expect, it } from "vitest";
import { safeText, validInput, MAX_INPUT } from "./presentation";
it("bounds input and rejects control characters and multiline paste", () => {
  expect(validInput('é 🦀 "hello"')).toBe(true);
  expect(validInput("a".repeat(MAX_INPUT))).toBe(true);
  expect(validInput("a".repeat(MAX_INPUT + 1))).toBe(false);
  for (const text of ["echo a\nexit", "\r", "\t", "\x1b", "\x00", "\x9b"])
    expect(validInput(text)).toBe(false);
});
it("neutralizes ANSI, OSC hyperlinks/clipboard, C1 and bidi without interpreting HTML", () => {
  const text = "\x1b]52;c;secret\x07\x1b[2J\x9b31m\u202e";
  const safe = safeText(text);
  expect(safe).toBe("\\u{1b}]52;c;secret\\u{7}\\u{1b}[2J\\u{9b}31m\\u{202e}");
  expect(safeText("<img src=x>\n\té 🦀")).toBe("<img src=x>\n\té 🦀");
  // Splitting an escape across writes cannot bypass the character-based policy.
  expect(safeText("\x1b") + safeText("[2J")).toBe("\\u{1b}[2J");
});
