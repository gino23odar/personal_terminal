import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import init, { WasmShell } from "../../../crates/gino-wasm/pkg/gino_wasm.js";

import wasmUrl from "../../../crates/gino-wasm/pkg/gino_wasm_bg.wasm?url";
import "@xterm/xterm/css/xterm.css";

export async function createRuntime(host: HTMLElement) {
  await init({ module_or_path: wasmUrl });
  const shell = new WasmShell();
  const terminal = new Terminal({
    disableStdin: true,
    convertEol: true,
    scrollback: 1000,
    cursorBlink: false,
    fontSize: 15,
    fontFamily: "monospace",
    theme: { background: "#0c1725", foreground: "#dae8ed", cursor: "#0c1725" },
  });
  const fit = new FitAddon();
  try {
    terminal.loadAddon(fit);
    terminal.open(host);
    fit.fit();
  } catch (error) {
    terminal.dispose();
    shell.free();
    throw error;
  }
  const resize = new ResizeObserver(() => {
    if (host.clientWidth > 0 && host.clientHeight > 0) fit.fit();
  });
  resize.observe(host);
  // Output is mirrored in a bounded plain-text log for assistive technology.
  host.setAttribute("aria-hidden", "true");
  terminal.textarea?.setAttribute("tabindex", "-1");
  return {
    shell,
    terminal,
    dispose() {
      resize.disconnect();
      shell.cancel();
      shell.free();
      terminal.dispose();
    },
  };
}
export type Runtime = Awaited<ReturnType<typeof createRuntime>>;
