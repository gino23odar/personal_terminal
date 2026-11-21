function equal(actual, expected) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      `Expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`,
    );
  }
}
function throwsBusy(fn) {
  try {
    fn();
  } catch (error) {
    if (/busy/.test(String(error))) return;
    throw error;
  }
  throw new Error("Expected busy error");
}
export const cases = {
  async commands(WasmShell) {
    const shell = new WasmShell();
    try {
      equal(shell.snapshot(), {
        cwd: "/home/visitor",
        lastStatus: 0,
        exitStatus: null,
      });
      for (const [input, text] of [
        ["echo 'hello 🦀' ''", "hello 🦀 \n"],
        ["pwd", "/home/visitor\n"],
        ["whoami", "visitor\n"],
        ['echo "$HOME"', "$HOME\n"],
      ]) {
        const result = await shell.execute(input);
        equal(result.status, 0);
        equal(result.error, null);
        equal(result.events, [{ type: "stdout", text }]);
      }
      const help = await shell.execute("help");
      equal(
        help.events[0].text
          .split("\n")
          .slice(1, -1)
          .map((line) => line.trim().split(" ")[0]),
        ["clear", "echo", "exit", "help", "pwd", "whoami"],
      );
      equal(
        (await shell.execute("help echo")).events[0].text.includes(
          "Usage: echo [--] [text ...]",
        ),
        true,
      );
      equal((await shell.execute("clear")).events, [{ type: "clearScreen" }]);
    } finally {
      shell.free();
    }
  },
  async errors(WasmShell) {
    const shell = new WasmShell();
    try {
      for (const [input, status, kind] of [
        ["missing", 127, "unknownCommand"],
        ["echo -n", 2, "invalidInput"],
        ["echo ok | whoami", 2, "syntax"],
        ["echo 'broken", 2, "syntax"],
      ]) {
        const result = await shell.execute(input);
        equal(result.status, status);
        equal(result.error.kind, kind);
        equal(result.events.length, 1);
        equal(result.events[0].type, "stderr");
        equal(result.session.lastStatus, status);
      }
      const syntax = await shell.execute("echo é | whoami");
      equal(syntax.error.span, [8, 9]);
      const blank = await shell.execute(" \t");
      equal(blank.status, 2);
      equal(blank.events, []);
      equal((await shell.execute("a".repeat(65537))).error.kind, "syntax");
      equal((await shell.execute("echo recovered")).status, 0);
    } finally {
      shell.free();
    }
  },
  async lifecycle(WasmShell) {
    const a = new WasmShell();
    const b = new WasmShell();
    try {
      equal((await a.execute("exit 7")).status, 7);
      equal(a.snapshot().exitStatus, 7);
      equal((await a.execute("echo ignored")).events, []);
      equal(b.snapshot().exitStatus, null);
      equal((await b.execute("whoami")).status, 0);
      a.reset();
      equal(a.snapshot(), {
        cwd: "/home/visitor",
        lastStatus: 0,
        exitStatus: null,
      });
      equal((await a.execute("echo reset")).events[0].text, "reset\n");
      const snapshot = a.snapshot();
      snapshot.cwd = "/changed";
      equal(a.snapshot().cwd, "/home/visitor");
    } finally {
      a.free();
      b.free();
    }
  },
  async cancellationAndBusy(WasmShell) {
    const shell = new WasmShell();
    try {
      equal(shell.cancel(), false);
      const pending = shell.execute("exit 7");
      throwsBusy(() => shell.execute("echo overlapping"));
      throwsBusy(() => shell.reset());
      throwsBusy(() => shell.snapshot());
      equal(shell.cancel(), true);
      const result = await pending;
      equal(result.status, 130);
      equal(result.error.kind, "cancelled");
      equal(result.events, []);
      equal(shell.snapshot().exitStatus, null);
      equal(shell.cancel(), false);
      equal((await shell.execute("echo fresh")).status, 0);
    } finally {
      shell.free();
    }
  },
  async freeWhileExecuting(WasmShell) {
    const shell = new WasmShell();
    const pending = shell.execute("echo retained");
    shell.free();
    equal((await pending).events, [{ type: "stdout", text: "retained\n" }]);
  },
};
