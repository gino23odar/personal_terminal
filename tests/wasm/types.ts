import {
  WasmShell,
  type ExecutionResult,
  type SessionSnapshot,
} from "../../crates/gino-wasm/pkg/gino_wasm.js";
const shell = new WasmShell();
const snapshot: SessionSnapshot = shell.snapshot();
const result: Promise<ExecutionResult> = shell.execute("help");
const cancelled: boolean = shell.cancel();
void [snapshot, result, cancelled];
// @ts-expect-error commands must be text
shell.execute(123);
// @ts-expect-error snapshot is typed, not any
void snapshot.nonexistent;
// @ts-expect-error result is typed, not any
result.then((value) => value.nonexistent);
