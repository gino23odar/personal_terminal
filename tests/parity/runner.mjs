// Runs the real generated WASM API; contains no expected shell behavior.
export async function runFixtures(WasmShell, fixtures) {
  const transcript = [];
  for (const fixture of fixtures) {
    const shell = new WasmShell();
    const steps = [];
    try {
      for (const step of fixture.steps) {
        if (step.reset) {
          shell.reset();
          steps.push({ session: shell.snapshot() });
        } else {
          const pending = shell.execute(
            step.input ?? step.repeat.repeat(step.count),
          );
          if (step.cancel) shell.cancel();
          steps.push(await pending);
        }
      }
      transcript.push({ name: fixture.name, steps });
    } finally {
      shell.free();
    }
  }
  return transcript;
}
export function expectedTranscript(fixtures) {
  return fixtures.map((fixture) => ({
    name: fixture.name,
    steps: fixture.steps.map((step) => step.expected),
  }));
}
