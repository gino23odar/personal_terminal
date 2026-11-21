export function App() {
  return (
    <main>
      <p className="eyebrow">PR-01 · Development scaffold</p>
      <h1>GINO.OS</h1>
      <p>
        A Rust-first virtual shell and engineering portfolio, in development.
      </p>
      <section aria-labelledby="status-heading">
        <h2 id="status-heading">Shell not connected yet</h2>
        <p>
          This page verifies the browser scaffold. No commands are available.
        </p>
        <p>
          The Rust core now implements six basic commands. The WebAssembly
          bridge and browser terminal interface are planned for later PRs.
        </p>
      </section>
    </main>
  );
}
