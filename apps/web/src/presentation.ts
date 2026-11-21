// Presentation limits are deliberately tighter than the core's 64 KiB parser cap.
export const MAX_INPUT = 4096;
export const MAX_HISTORY = 100;

export function validInput(text: string): boolean {
  return (
    text.length <= MAX_INPUT &&
    !Array.from(text).some((char) => {
      const code = char.codePointAt(0)!;
      return code < 32 || (code >= 127 && code <= 159);
    })
  );
}

// No command-controlled escape sequence reaches xterm's parser. Only LF and
// tabs survive; control bytes and bidi overrides are printed visibly.
export function safeText(text: string): string {
  return Array.from(text, (char) => {
    const code = char.codePointAt(0)!;
    if (char === "\n" || char === "\t") return char;
    if (
      code < 32 ||
      (code >= 127 && code <= 159) ||
      (code >= 0x202a && code <= 0x202e) ||
      (code >= 0x2066 && code <= 0x2069)
    ) {
      return `\\u{${code.toString(16)}}`;
    }
    return char;
  }).join("");
}
