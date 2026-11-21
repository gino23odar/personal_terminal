import { useEffect, useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import type { Runtime } from "./runtime";
import { MAX_HISTORY, MAX_INPUT, safeText, validInput } from "./presentation";

const CLEAR = "\x1b[2J\x1b[3J\x1b[H"; // Trusted UI control, never command text.
const LOG_LIMIT = 32768;

export function App() {
  const [generation, setGeneration] = useState(0);
  return (
    <main>
      <header>
        <p className="eyebrow">GINO.OS / VIRTUAL SHELL</p>
        <h1>Your browser. A Rust shell.</h1>
        <p>
          Type <code>help</code> to explore the available commands.
        </p>
      </header>
      <BrowserShell
        key={generation}
        retry={() => setGeneration((value) => value + 1)}
      />
      <footer>
        Development preview · Filesystem and portfolio commands are still
        planned.
      </footer>
    </main>
  );
}

function BrowserShell({ retry }: { retry: () => void }) {
  const host = useRef<HTMLDivElement>(null);
  const input = useRef<HTMLInputElement>(null);
  const runtime = useRef<Runtime | null>(null);
  const alive = useRef(false);
  const active = useRef(false);
  const history = useRef<string[]>([]);
  const position = useRef(0);
  const draft = useRef("");
  const [phase, setPhase] = useState<
    "loading" | "ready" | "busy" | "closed" | "failed"
  >("loading");
  const [line, setLine] = useState("");
  const [status, setStatus] = useState(0);
  const [cwd, setCwd] = useState("/home/visitor");
  const [notice, setNotice] = useState("");
  const [log, setLog] = useState("");

  function print(text: string) {
    runtime.current?.terminal.write(text);
    setLog((previous) => (previous + text).slice(-LOG_LIMIT));
  }

  useEffect(() => {
    let disposed = false;
    alive.current = true;
    void import("./runtime")
      .then(async ({ createRuntime }) => {
        if (disposed) return;
        const instance = await createRuntime(host.current!);
        if (disposed) {
          instance.dispose();
          return;
        }
        runtime.current = instance;
        const greeting =
          "GINO.OS — browser virtual shell\nType 'help' for available commands.\n";
        instance.terminal.write(greeting);
        setLog(greeting);
        setPhase("ready");
        input.current?.focus();
      })
      .catch(() => {
        if (!disposed) {
          setPhase("failed");
          setNotice(
            "Could not load the shell. Check your connection and retry.",
          );
        }
      });
    return () => {
      disposed = true;
      alive.current = false;
      runtime.current?.dispose();
      runtime.current = null;
    };
  }, []);

  async function execute(command: string) {
    const instance = runtime.current;
    if (!instance || active.current || phase !== "ready") return;
    if (!validInput(command)) {
      setNotice(
        `Use a single line of at most ${MAX_INPUT} characters without control characters.`,
      );
      return;
    }
    active.current = true;
    setPhase("busy");
    setNotice("");
    setLine("");
    if (command.trim()) {
      history.current = [...history.current, command].slice(-MAX_HISTORY);
    }
    position.current = history.current.length;
    draft.current = "";
    print(`visitor@gino.os:${safeText(cwd)}$ ${safeText(command)}\n`);
    try {
      const result = await instance.shell.execute(command);
      if (!alive.current) return;
      for (const event of result.events) {
        if (event.type === "clearScreen") {
          instance.terminal.write(CLEAR);
          setLog("");
        } else {
          const text = safeText(event.text);
          instance.terminal.write(
            event.type === "stderr" ? `\x1b[31m${text}\x1b[0m` : text,
          );
          setLog((previous) => (previous + text).slice(-LOG_LIMIT));
        }
      }
      setStatus(result.status);
      setCwd(result.session.cwd);
      setPhase(result.session.exitStatus === null ? "ready" : "closed");
      if (result.session.exitStatus !== null)
        setNotice(
          `Session closed with status ${result.status}. Start a new session to continue.`,
        );
      else if (result.error?.kind === "cancelled")
        setNotice("Command cancelled.");
    } catch {
      if (alive.current) {
        setPhase("failed");
        setNotice(
          "The shell stopped unexpectedly. Reload the shell to continue.",
        );
      }
    } finally {
      active.current = false;
      if (alive.current) input.current?.focus();
    }
  }

  function reset() {
    if (active.current || !runtime.current) return;
    runtime.current.shell.reset();
    runtime.current.terminal.write(CLEAR);
    history.current = [];
    position.current = 0;
    draft.current = "";
    setLine("");
    setLog("");
    setStatus(0);
    setCwd(runtime.current.shell.snapshot().cwd);
    setNotice("");
    setPhase("ready");
    input.current?.focus();
  }

  function keyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.nativeEvent.isComposing) return;
    if (event.ctrlKey && event.key.toLowerCase() === "c") {
      // Preserve ordinary selection/copy in the input.
      if (input.current?.selectionStart !== input.current?.selectionEnd) return;
      event.preventDefault();
      if (active.current) runtime.current?.shell.cancel();
      else {
        setLine("");
        setNotice("Input cancelled.");
      }
    } else if (
      event.ctrlKey &&
      event.key.toLowerCase() === "d" &&
      line === ""
    ) {
      event.preventDefault();
      void execute("exit");
    } else if (event.key === "Escape") {
      event.preventDefault();
      setLine("");
    } else if (
      phase === "ready" &&
      !event.ctrlKey &&
      !event.metaKey &&
      !event.altKey &&
      (event.key === "ArrowUp" || event.key === "ArrowDown")
    ) {
      event.preventDefault();
      if (position.current === history.current.length) draft.current = line;
      position.current = Math.max(
        0,
        Math.min(
          history.current.length,
          position.current + (event.key === "ArrowUp" ? -1 : 1),
        ),
      );
      setLine(history.current[position.current] ?? draft.current);
    }
  }

  return (
    <section className="shell" aria-label="Virtual shell">
      <div className="shell-bar">
        <span role="status">
          {phase === "loading"
            ? "Loading shell…"
            : phase === "ready"
              ? "Ready"
              : phase === "busy"
                ? "Running…"
                : phase === "closed"
                  ? "Session closed"
                  : "Shell unavailable"}
        </span>
        <span>Exit status: {status}</span>
      </div>
      <div ref={host} className="terminal" />
      <pre
        className="sr-only"
        role="log"
        aria-label="Terminal output"
        aria-live="polite"
        aria-relevant="additions text"
      >
        {log}
      </pre>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void execute(line);
        }}
      >
        <label htmlFor="command">
          visitor@gino.os:{cwd === "/home/visitor" ? "~" : cwd}$
        </label>
        <input
          id="command"
          ref={input}
          aria-label="Command"
          aria-describedby="keys"
          autoComplete="off"
          spellCheck={false}
          value={line}
          maxLength={MAX_INPUT}
          readOnly={phase !== "ready"}
          onChange={(event) => {
            if (validInput(event.target.value)) {
              setLine(event.target.value);
              setNotice("");
            } else setNotice("Control characters are not accepted.");
          }}
          onKeyDown={keyDown}
          onPaste={(event) => {
            const text = event.clipboardData.getData("text");
            const selected =
              (input.current?.selectionEnd ?? 0) -
              (input.current?.selectionStart ?? 0);
            if (
              !validInput(text) ||
              line.length - selected + text.length > MAX_INPUT
            ) {
              event.preventDefault();
              setNotice(
                "Paste rejected: use one line, no control characters, and at most 4096 characters.",
              );
            }
          }}
        />
        <button disabled={phase !== "ready"}>Run</button>
      </form>
      <p id="keys" className="hint">
        Enter to run · ↑/↓ history · Ctrl+C cancel · Ctrl+D exit on empty input
      </p>
      {notice && <p role="alert">{notice}</p>}
      {phase === "closed" && <button onClick={reset}>New session</button>}
      {phase === "failed" && <button onClick={retry}>Reload shell</button>}
    </section>
  );
}
