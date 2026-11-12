# CODEX MASTER PROMPT — GINO.OS (RUST-FIRST, PR-DRIVEN)

## 0. Your role and execution rules

Act as a principal Rust systems engineer, shell/runtime architect, full-stack engineer, and technical lead. Design, implement, test, document, and progressively ship **GINO.OS**, a genuine cross-platform **virtual shell**, interactive engineering portfolio, and extensible terminal playground.

**This is an executable product specification and a roadmap, not an instruction to build everything in one run.** Read the entire specification before designing anything. Inspect the repository before modifying files. Make reasonable technical decisions, record tradeoffs, prefer working vertical slices to scaffolding, and never fabricate build results, employment claims, features, or tests.

### Non-negotiable execution contract

- **Rust owns the shell:** parser, AST, interpreter, command registry, session, virtual filesystem, standard and custom commands, and animation/application contracts must be written in Rust. The native Linux application is written in Rust, not Node.js.
- **One shared Rust implementation:** the same portable Rust shell core must power the native CLI and the browser via **WebAssembly**. Do not create a TypeScript reimplementation of shell behavior.
- **TypeScript is only the web UI/glue:** React, Vite, and `@xterm/xterm` handle presentation, accessibility, and browser integration. They do not interpret commands or implement virtual filesystem semantics.
- **Backend remains separately deployable:** the optional AI service may use Python/FastAPI; it is not the shell or terminal execution engine.
- **No host-shell passthrough:** neither the public browser nor the native virtual shell may automatically execute entered commands through Bash, Zsh, `std::process::Command`, or a production container.
- **PR-by-PR delivery:** use the 36 ordered PRs below as the implementation timeline. Each PR has focused scope, tests, a reviewable diff, and a completion gate. Do not bundle future PRs without explicit authorization.
- **Initial Codex execution is PR-01 only.** Implement PR-01, verify it, report results, and stop. The full PR schedule must still be documented. If the repository already contains PR-01 work, identify the first uncompleted PR and ask for approval before changing the intended scope.
- Do not claim access to GitHub PR creation unless the environment actually provides it. If PR creation is unavailable, implement the current unit on a feature branch when possible, and supply a PR title, description, changed files, verification evidence, and suggested commit message.

## 1. Product vision

GINO.OS is a Linux-inspired interactive terminal that operates in:

1. **Native Linux CLI:** a Rust executable running inside an ordinary terminal emulator, completely independent of the browser or backend for local commands.
2. **Browser:** a React/xterm.js frontend calling the same Rust shell compiled to WASM; local shell commands execute in the visitor's browser.
3. **Hosted deployment:** independently deployed static frontend and optional AI API, with a custom domain such as `terminal.ginoodar.com`.

It combines four capabilities:

- **Real virtual shell:** documented Unix-like grammar and commands, asynchronous I/O, session state, virtual filesystem, completion and keyboard handling.
- **Engineering portfolio:** verified project, experience, skills, architecture and contact information accessible as commands and virtual files.
- **Terminal entertainment:** independently pluggable funny commands, original ASCII animation, Easter eggs, and at least one keyboard-controlled game.
- **Portfolio AI:** an optional backend-assisted `ask` command for source-grounded professional information.

Its primary users are recruiters, engineering interviewers, curious developers, and the owner maintaining the application. It must be technically authentic yet approachable to someone who does not know Unix commands.

**Product priority:** functional extensible shell first; genuinely useful portfolio second; polished entertainment third; AI enhancement fourth. A visually impressive façade is not sufficient.

## 2. Existing context and accuracy rules

The owner maintains an existing GitHub profile at `github.com/gino23odar`, branded **GINO.OS**, with dark-navy, cyan/teal, monospaced terminal artwork, animated ASCII elements, collapsible command navigation, and an Issue-based portfolio Q&A. Its structured sources may include `config/profile_knowledge.json`, `config/projects.json`, public project metadata, and a featured-project generator. These may live **in a different repository**; do not assume they exist locally or alter the profile repo without permission.

The public professional story includes backend and API integration work, Java/Spring Boot, Playwright QA and E2E testing, cross-layer debugging, reporting systems, production monitoring, and tools such as Kafka, PostgreSQL, OpenSearch, ClickHouse, Docker, Linux, Jenkins, Semaphore CI, New Relic, and CloudWatch. The AI direction includes applied AI, local LLMs, coding agents, MCP, retrieval, orchestration, and evaluation. Differentiate **proven professional experience**, **personal/academic projects**, **experiments**, and **currently explored tools**.

Use approved, version-controlled content only. Never invent employers, certifications, shipped systems, performance metrics, or customer outcomes. Never publish proprietary code or private workplace details. Where data is missing, use clearly marked placeholders rather than plausible-sounding inventions.

## 3. Technology and portability decisions

### Rust shared core

Prefer a Rust workspace with small crates, for example:

- `gino-shell`: parser, AST, interpreter, session, command registry, execution contracts.
- `gino-vfs`: portable virtual filesystem model and storage interfaces (introduced only when needed).
- `gino-commands`: built-in and portfolio commands (can start inside `gino-shell` if smaller is cleaner).
- `gino-animation`: portable animation/application abstractions (deferred).
- `gino-wasm`: `wasm-bindgen` boundary exposing the shared Rust shell to the browser.
- `gino-cli`: native Rust executable and terminal adapter.

Do **not** create unused crates or circular dependencies. A minimal workspace in PR-01 is preferred.

The shared Rust crates must compile for **native Linux** and **`wasm32-unknown-unknown`** as appropriate. Avoid OS-only APIs in the shared core (`std::fs` host access, process execution, terminal raw mode, OS environment assumptions). Place platform-specific concerns behind traits/adapters and use conditional compilation only where necessary. Decide and document a consistent strategy for asynchronous commands on native and single-threaded WASM (e.g., portable futures and a carefully designed command-handler abstraction). Do not assume all `Send` constraints automatically work in WASM.

### Native terminal

Rust stable toolchain; use a maintained line editor such as `reedline`/`rustyline` if appropriate, and `crossterm` when low-level terminal events, resizing, or animations are necessary. Keep terminal raw-mode ownership and cleanup in the native adapter. A full-screen TUI framework is optional; **do not force fullscreen** just to show a shell prompt.

### Browser

React + TypeScript + Vite + `@xterm/xterm`, pnpm for the web workspace, and `wasm-bindgen`/`wasm-pack` or another documented, maintained build pipeline. The browser sends keystrokes/commands to Rust WASM and renders returned output/events. Do not hardcode command behavior in React. Use IndexedDB later for browser workspace persistence; no backend round trips for ordinary local commands.

### AI backend (later)

Python/FastAPI + Pydantic + Pytest, provider abstraction, approved portfolio retrieval, HTTP API, streaming where useful, and explicit security/cost controls. No AI backend is required to launch the native CLI or the basic browser shell.

### Tooling

`cargo fmt`, `cargo clippy`, `cargo test`, Rust and WASM build checks, browser tests (Vitest and Playwright), GitHub Actions, Docker/Compose for later deployment. Select maintained versions that work together and document pinning/toolchain choices; never guess that a package API exists.

## 4. Suggested repository structure

```text
gino-os/
├── crates/
│   ├── gino-shell/              # portable Rust shell core
│   ├── gino-cli/                # native Linux CLI adapter/executable
│   ├── gino-wasm/               # Rust WASM bridge
│   ├── gino-vfs/                # added at PR-10
│   └── gino-animation/          # added at PR-25
├── apps/
│   ├── web/                     # React/Vite/xterm.js rendering only
│   └── api/                     # FastAPI; added at PR-29
├── knowledge/                   # approved public portfolio facts
├── docs/
│   ├── PRODUCT_SPEC.md
│   ├── IMPLEMENTATION_PLAN.md
│   ├── ARCHITECTURE.md
│   ├── COMMANDS.md
│   ├── EXTENDING.md
│   └── DEPLOYMENT.md
├── infrastructure/
├── .github/workflows/
├── Cargo.toml
├── rust-toolchain.toml
├── pnpm-workspace.yaml
└── README.md
```

Create folders only when relevant. Preserve any existing repository organization if it already achieves the same separation.

## 5. Rust shell execution architecture

Define explicit portable contracts before implementing commands. The following interfaces are conceptual, not an instruction to use this exact Rust syntax or prematurely freeze public APIs:

- **`ShellSession`** owns virtual current directory, environment, aliases, history metadata, and command lifecycle. Clearly distinguish mutation of session state from a borrowed execution snapshot.
- **`CommandRegistry`** maps command names/aliases to metadata and handlers; powers `help`, discovery, and future completion. No giant match/switch dispatcher.
- **`CommandContext`** exposes only approved capabilities (virtual filesystem, portfolio reader, cancellation, random source, animation authority, AI client when enabled). No unrestricted OS references.
- **`CommandIo`** provides asynchronous stdin, stdout, stderr, and an output/event contract with bounded buffers or backpressure for pipes/large output.
- **`CommandResult`** conveys exit status and structured errors; map failures to clear terminal feedback.
- **Cancellation** is per-execution and must propagate to streams, animations, games, and network calls. Use portable primitives with consistent semantics in both runtimes.
- **`TerminalAdapter`** owns presentation, terminal dimensions, keyboard events, clear-screen control, raw-mode lifecycle, and user-facing output. The shell engine must not depend on `crossterm`, xterm.js, or the DOM.
- **`InteractiveAppSession`** (later) temporarily obtains exclusive keyboard input and frame drawing, then releases them on completion, interruption, or panic/error.

Keep command execution asynchronous-capable without making every basic operation unnecessarily complicated. Preserve a migration path to byte streams/pipelines. For WASM avoid patterns that require threads, process spawning, or native blocking I/O. Web-boundary calls should expose typed result/output events or a documented async interface, not raw mutable internal Rust objects.

## 6. Shell parser: real grammar, progressive features

Implement tokenization and an AST (or equivalent structured syntax tree). Do not use a sequence of brittle string comparisons or regexes as the shell interpreter.

- **PR-04 / V0.1:** simple command; whitespace; single/double quoted arguments; defined escape behavior; informative malformed-quote errors. Recognize but reject unsupported operators rather than pretending to execute them.
- **PR-15:** environment-variable and home expansion; aliases; clearly specified expansion order.
- **PR-16:** pipelines with stream direction, cancellation and exit-status semantics.
- **PR-17:** redirection (`<`, `>`, `>>`), sequences (`;`), conditional chaining (`&&`, `||`).
- **Beyond V1:** globbing, functions, subshells, scripting, richer POSIX compatibility only if justified.

Document the supported grammar, error model, quoting, expansion precedence, and shell compatibility limits. Example target interactions:

```sh
ls -la
cd ~/projects
cat README.md
cat skills.txt | grep -i python > matches.txt
echo "hello world" >> notes.txt
mkdir -p workspace/notes
cat notes.txt
```

Do not advertise redirection or other syntax as available before the corresponding PR lands.

## 7. Virtual filesystem requirements (PRs 10–14)

The shell must eventually expose a virtual Linux-like tree:

```text
/
├── bin/
├── etc/
│   ├── hostname
│   └── os-release
├── home/visitor/
│   ├── README.md
│   ├── projects/
│   ├── experience/
│   ├── skills/
│   └── workspace/        # writable
├── tmp/
└── usr/share/gino-os/
```

Support portable path resolution, files/directories, metadata, read-only portfolio mount, isolated writable workspace, reset, bounds/quotas, and persistence. Implement an in-memory storage backend first; later adapt IndexedDB for the browser and app-data storage for the CLI. Browser sessions must not share writable state. Do not leak host paths, traverse outside virtual mounts, or silently turn virtual paths into host file operations.

Implement documented Unix-like commands incrementally:

- Navigation: `pwd`, `ls`, `cd`, `tree`.
- File operations: `touch`, `mkdir`, `rmdir`, `cp`, `mv`, `rm`.
- Inspection: `cat`, `head`, `tail`, `less`.
- Processing: `echo`, `grep`, `wc`, `sort`, `uniq`.
- Environment: `env`, `export`, `printenv`, `alias`, `unalias`.
- Virtual system: `whoami`, `hostname`, `uname`, `date`, `uptime`.
- Utilities: `help`, `man`, `history`, `clear`, `which`, `exit`.

Do not silently accept unsupported flags; document divergences from GNU/POSIX utilities. `whoami`, `uname`, etc. identify the **virtual environment**, not the host or user account. Basic `pwd` in PR-05 may use an initial virtual working-directory abstraction before the full VFS exists.

## 8. Portfolio: one approved source of truth (PRs 20–24)

Implement commands: `profile`, `experience`, `projects`, `stack`, `ai`, `architecture`, `contact`, and later `ask`. Keep `whoami` compatible with normal virtual-shell meaning. The same portfolio records must generate both command output and read-only virtual files (`~/projects/.../README.md`, `~/experience/overview.md`, etc.).

Use a typed schema, evidence/public links, tags, stable IDs, and explicit status metadata where useful. Integrate the existing GitHub-profile facts only after verifying that they are public and approved. Avoid duplicating their content in multiple UI components. Make project selection/hot-swapping possible by editing structured data rather than rewriting commands.

### Recruiter UX

Provide safe quick-launch buttons/command palette for visitors unfamiliar with shell commands. These must call the **same Rust command registry**, not a parallel React-only implementation. Support curated, shareable deep links like `/?command=projects` or `/?project=queue-management` with allowlisted read-only operations; **never auto-execute arbitrary URL-provided shell strings**. Provide repository links, project summaries, accessible alternative navigation, keyboard-first use, responsive layout and reduced motion. Public portfolio browsing must work with the AI API offline.

## 9. ASCII animations and games (PRs 25–28)

Create a portable frame-based/procedural animation runtime using terminal adapter capabilities, ANSI styling, terminal dimensions, bounded frame rate, in-place updates, cancellation, pause/resume where suitable, reduced motion, cleanup, and CPU/memory limits. Avoid flooding scrollback.

Target independent fun commands: `fortune`, `cowsay`, `coffee`, `matrix`, `doggo`, `catwalk`, `sl`, `rain`, `hack` (clearly fictional), and `fun`. Use original or properly licensed artwork. Implement at least one small keyboard-controlled game (e.g., original Snake) that acquires and releases terminal input/rendering safely in both native and browser versions. Show how to add a command without touching the interpreter.

## 10. Portfolio-aware AI (PRs 29–33)

`ask "What experience does Gino have with Playwright?"` should call an optional, separately hosted FastAPI API to retrieve approved records and generate evidence-grounded answers with source links. Start with structured/keyword retrieval; no vector DB until scale or relevance requires it. Support a replaceable model provider, errors, cancellation and optional streaming. If information is missing, say so rather than inventing credentials, experience or shipped architectures.

Keep model credentials server-side; constrain AI tools to an explicit allowlist of **public portfolio retrieval only**. Protect against prompt injection from questions/docs; validate inputs; enforce rate, request-size, timeout, token and spending budgets; privacy-conscious logs; avoid leaking prompts/secrets. No unrestricted agent execution of either virtual write commands or production host commands. All ordinary local shell commands continue functioning without the API.

MCP interoperability, tool-calling orchestration, agent workflows and evaluations may become documented future integrations or demonstrations, but they are **not proof of production experience** and are not prerequisites for initial releases.

## 11. Security, reliability and accessibility gates

- No host-shell passthrough or host filesystem exposure.
- Portable VFS path boundaries, read-only mounts, workspace quotas, and per-session isolation.
- Browser output treated as untrusted; safely handle ANSI sequences, terminal hyperlinks and text; do not execute generated HTML.
- URL deep links limited to approved read-only actions.
- No browser/API secrets, unrestricted model/tool calls, or unlimited AI spend.
- Prompt interruption and animation/game cleanup, including restoration of native terminal state.
- Basic commands and portfolio remain functional during API failures.
- Keyboard navigation, good contrast, reduced motion, screen-reader labels, mobile alternatives, and fast/skippable boot.
- Type checking, Rust linting, automated tests and build gates at each PR.

Document trust boundaries for browser, WASM bridge, browser storage, backend, approved knowledge, external model provider and host infrastructure before enabling public AI access.

## 12. Test and verification strategy

Test behavior, not implementation trivia. Required categories as features arrive:

- Rust parser/AST/quoted-string/invalid-syntax tests.
- Registry, command execution, exit-status, cancellation and session tests.
- VFS properties: paths, isolation, read-only, quotas, persistence, reset.
- Pipelines, redirects, environment expansion and text utility behavior.
- Native CLI integration tests with terminal input/output where practical.
- Rust/WASM parity tests using representative command fixtures and comparable results.
- React/xterm.js browser integration and Playwright E2E tests.
- Animation terminal-resize, cancellation, reduced-motion and cleanup tests.
- FastAPI endpoint, retrieval, grounding, abuse-budget and offline-fallback tests.
- Security regressions including unsafe paths, injection attempts, escape sequences and unintended host execution.

Minimum applicable Rust checks: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and build/check for the relevant native/WASM targets. Run WASM browser tests or a browser E2E smoke test whenever the bridge changes. Run frontend type checks, tests and production build when web files change. Python backend checks begin only when that app exists. If a check cannot run, state exactly why; never claim it passed.

## 13. UX and boot behavior

Use existing GINO.OS aesthetics: dark navy/black, restrained cyan/teal, monospaced typography, clear prompts, readable output, subtle retro styling, and smooth accessible animations. Do not repeat the README's animation as heavy UI decoration at the expense of performance.

Example boot **only when those components truly exist**:

```text
GINO.OS v0.1.0
[ OK ] Loading shell core
[ OK ] Registering commands
Welcome to GINO.OS
Type 'help' for available commands.
visitor@gino.os:~$
```

Boot should be fast and skippable. Do not advertise unimplemented commands in help text. Standard commands and their documented behavior must take precedence over easter eggs.

## 14. Release checkpoints (no dates or duration estimates)

- **V0.1 / after PR-09:** native Rust CLI and browser terminal use the same Rust commands through a WASM adapter; basic shell features tested. Architectural proof.
- **V0.5 / after PR-24:** functional virtual filesystem, shell composition and useful public portfolio with guided exploration; deploy a static preview. Recruiter-friendly preview.
- **V1.0 / after PR-36:** polished shell, public portfolio, fun command/game runtime, secure optional AI, tested deployment, documentation and release notes.

AI may be left disabled at release if credentials/security/budgeting are not production-ready; explicitly mark its status. Do not declare a milestone complete based on visual screenshots alone.

## 15. Implementation timeline — 36 focused pull requests

**This is the timeline. Use PR sequence and dependency gates, not dates, story points, person-hours, or estimated durations.** `PR-XX` is the *planned sequence*, not a claim that a GitHub PR already exists. Feature branches should use `feat/pr-XX-short-name` or equivalent. Each PR must be independently reviewable, pass relevant checks, add tests for behavior, and update documentation where needed.

### Phase 1 — Rust shell foundation (PR-01 → PR-09) · V0.1 gate

| PR | Scope | Definition of done |
|---|---|---|
| **PR-01** | Initialize Rust workspace, minimal CLI crate and web scaffold, toolchains, formatting/lint/test scripts, PR template, CI baseline | Repo installs/builds in documented environment; baseline checks execute; architecture boundaries and PR plan recorded |
| **PR-02** | Portable shell contracts: session, command/context, I/O/events, cancellation, adapter traits | Contracts compile natively and for WASM target; unit/compile checks demonstrate boundaries |
| **PR-03** | Rust command registry, metadata, aliases, lookup and help integration | Register/lookup/alias tests pass; no monolithic dispatcher |
| **PR-04** | Rust tokenizer, minimal AST, quotes/escapes, malformed syntax errors | Parser tests cover valid/invalid input; unsupported operators rejected |
| **PR-05** | Rust interpreter, session lifecycle and `help`, `echo`, `pwd`, `whoami`, `clear`, `exit` | Built-ins execute through registry with stable output/exit codes |
| **PR-06** | Native Rust Linux CLI adapter with history, input, Ctrl+C/D behavior, clean shutdown | Interactive Linux command sequence works without browser/server; terminal state restored |
| **PR-07** | Rust WASM adapter/bridge exposing same shell engine | WASM build passes; JS-facing tests prove commands come from Rust core |
| **PR-08** | React/Vite/xterm.js browser terminal, keyboard handling and Rust WASM binding | Browser executes shared commands locally; no duplicate TS interpreter |
| **PR-09** | Cross-runtime parity fixtures, CLI/browser smoke tests, docs and V0.1 audit | Native + WASM outputs equivalent for fixtures; checks green; V0.1 criteria evidenced |

### Phase 2 — Virtual filesystem (PR-10 → PR-14)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-10** | VFS Rust types, in-memory backend, filesystem metadata and service contracts | Portable operations compile for native/WASM; unit tests pass |
| **PR-11** | Path resolution, read-only mounts, permissions/capabilities, quotas | Traversal, mount and isolation regression tests pass |
| **PR-12** | `ls`, `cd`, `cat`, `mkdir`, `touch`, `cp`, `mv`, `rm`, `tree` plus appropriate utilities | Consistent command behavior and explicit unsupported-option errors |
| **PR-13** | Per-session persistence adapters: browser IndexedDB and native app-data storage | Data survives restart when enabled, reset works, no host path leak |
| **PR-14** | VFS parity, state-invariant tests and documentation | Same file-operation fixtures pass natively and in browser |

### Phase 3 — Shell composition (PR-15 → PR-19)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-15** | Variable/home expansion, aliases, virtual `env`/`export`/`printenv` | Expansion order documented; quoting and state tests pass |
| **PR-16** | Portable asynchronous pipes/stream forwarding, backpressure, cancellation | Multi-stage pipeline tests pass in CLI and WASM browser |
| **PR-17** | Input/output redirection, append, `;`, `&&`, `||` | AST/operator precedence and exit-code semantics tested |
| **PR-18** | Text utilities `grep`, `head`, `tail`, `wc`, `sort`, `uniq` and documented flags | Utilities compose through pipes and redirects; errors meaningful |
| **PR-19** | Advanced shell conformance fixtures and hardening | Full documented examples work across runtimes; regression suite passes |

### Phase 4 — Public portfolio (PR-20 → PR-24) · V0.5 gate

| PR | Scope | Definition of done |
|---|---|---|
| **PR-20** | Canonical approved portfolio schema, import/validation tools | Facts provenance and statuses preserved; no fabricated data |
| **PR-21** | Read-only virtual portfolio tree generated from canonical data | Data appears correctly under virtual files; writes blocked |
| **PR-22** | `profile`, `experience`, `projects`, `stack`, `ai`, `architecture`, `contact` commands | Output consistent with virtual files and public links |
| **PR-23** | Recruiter shortcuts, command palette, safe deep links, mobile/accessibility improvements | Nontechnical visitor can explore; URL commands allowlisted |
| **PR-24** | Portfolio E2E, content accuracy audit, deployable preview, V0.5 documentation | Hosted preview works without AI; V0.5 criteria evidenced |

### Phase 5 — Animations and extension SDK (PR-25 → PR-28)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-25** | Portable animation runtime, frame lifecycle, reduced motion, cancellation | Animation works in both adapters and restores terminal on interruption |
| **PR-26** | Curated independent ASCII/fun commands with original assets | New animations use shared runtime; no scrollback flooding |
| **PR-27** | One interactive game (e.g., Snake) with temporary keyboard ownership | Controls, resizing, pause/exit and cleanup tested |
| **PR-28** | Extensibility documentation/SDK example and plugin boundary tests | New custom command/game added without interpreter modification |

### Phase 6 — Optional AI service (PR-29 → PR-33)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-29** | FastAPI skeleton, typed schemas, health/error endpoints, config | API checks and tests pass independently of CLI/web |
| **PR-30** | Retrieval from approved portfolio records with evidence links | Grounded retrieval tested; unknown facts handled gracefully |
| **PR-31** | Provider abstraction and grounded answer-generation/streaming | Swappable providers; no model keys in client; mock tests pass |
| **PR-32** | Rate/quota/spend budgets, timeouts, prompt-injection defenses, safe logging | Abuse/security tests pass; bounded failure modes documented |
| **PR-33** | Rust `ask` command through optional HTTP client capability, CLI/web UX and offline errors | Same command works across runtimes; cancellation and fallback tested |

Phases 5 and 6 can proceed on separate branches after Phase 4 if contracts are stable. Merge conflict-sensitive integration work should be sequenced; PR numbers are review-order identifiers, **not elapsed-time estimates**.

### Phase 7 — Production and V1.0 (PR-34 → PR-36)

| PR | Scope | Definition of done |
|---|---|---|
| **PR-34** | Production Docker/API deployment and static frontend hosting, TLS/config/health docs | Reproducible staging deployment; frontend and API independently configurable |
| **PR-35** | CI/CD integration, E2E/accessibility/security/load/failure validation | Release gates green; known issues documented and blocking ones fixed |
| **PR-36** | Installation, architecture, command catalog, contributing/extension docs, release notes, GitHub profile links | Public demo and native install verified; V1.0 checklist complete |

### PR workflow and merge gates

For **every PR**:

1. Read `docs/PRODUCT_SPEC.md` and `docs/IMPLEMENTATION_PLAN.md`; inspect existing code and previous PR outcomes.
2. State **scope, non-goals, dependencies, architecture impact, acceptance criteria**.
3. Implement the smallest complete vertical slice; avoid speculative code.
4. Add or update automated tests and run applicable native/WASM/web/API checks.
5. Review security, portability, cancellation, docs, and regressions.
6. Submit a clear PR description: summary, screenshots/CLI output where useful, verification commands/results, limitations, follow-ups.
7. Update roadmap status only after acceptance; do not mark a PR complete before checks pass.

Use a PR template with checkboxes for scoped change, user-visible behavior, test evidence, Rust native/WASM parity where applicable, security review, documentation, and unresolved concerns. No PR merges with known failing mandatory checks. If an external service blocks validation, describe the limitation rather than fabricate green results.

## 16. Documentation and open-source quality

Maintain accurate `README.md`, `docs/PRODUCT_SPEC.md`, `docs/IMPLEMENTATION_PLAN.md` (with PR statuses), `ARCHITECTURE.md`, `COMMANDS.md`, `EXTENDING.md`, `DEPLOYMENT.md`, and ADRs for major decisions. Include a LICENSE and CONTRIBUTING guidance before broad public collaboration. Documentation and boot/help UI must only advertise implemented commands. Provide terminal screenshots/GIF demos as optional polish, not substitutes for tests.

Document local developer commands as they become valid, e.g.:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p gino-cli
# After PR-07/08:
# wasm-pack build crates/gino-wasm --target web
# pnpm --filter @gino-os/web dev
```

Use actual crate/package names defined in PR-01; keep these commands synchronized with the repository. Web tooling must build the WASM artifact reproducibly and must not depend on committing machine-specific binaries.

## 17. Scope control and PR-01 immediate deliverables

### Step A — Inspect

Read existing project structure, source code, README, tests, build scripts, installed toolchains, and public assets; do not overwrite useful work. Inspect whether the separate profile data is accessible; do not assume external repository permissions.

### Step B — Choose the smallest viable layout

Propose and document a concise Rust workspace architecture, `gino-cli`, `gino-shell`, planned `gino-wasm`, and web scaffold boundaries. Identify the initial crate targets and test commands. Add a simple dependency diagram and explain why Rust + WASM is the portability strategy.

### Step C — Implement **PR-01 only**

- Initialize/normalize the Rust workspace and native CLI entry point **without inventing the shell implementation yet**.
- Set up Rust toolchain configuration, formatting, clippy, unit-test scripts, and a minimal passing crate/test smoke check.
- Initialize the Vite/React/TypeScript web scaffold or integrate an existing one, with a placeholder that clearly states the shell is not connected yet.
- Add documented scripts for the Rust/WASM build pipeline as appropriate, but do not claim the WASM bridge works before PR-07.
- Add GitHub Actions baseline CI for the components currently present.
- Create `docs/PRODUCT_SPEC.md`, `docs/IMPLEMENTATION_PLAN.md` with the complete 36-PR plan, and initial `ARCHITECTURE.md`/README run instructions.
- Ensure basic builds/tests run; avoid empty placeholder packages beyond imminent PR boundaries.

### Step D — Verify and report

Run all available checks that apply to PR-01. Show exact commands and results. If a command cannot run, say why. End with:

1. PR title and description.
2. Files created/modified.
3. Architectural decisions.
4. Build/test/lint outcomes.
5. Known limitations.
6. PR-01 acceptance criteria status.
7. The concrete scope and prerequisites of PR-02.

**Stop. Do not start PR-02 or pretend that PR-01 already implements the shared shell.**

## 18. End-state demonstration (not current functionality)

The finished system should support examples such as:

```text
visitor@gino.os:~$ help
visitor@gino.os:~$ ls
visitor@gino.os:~$ cd projects
visitor@gino.os:~/projects$ cat README.md
visitor@gino.os:~$ architecture
visitor@gino.os:~$ matrix
visitor@gino.os:~$ snake
visitor@gino.os:~$ ask "What experience does Gino have with distributed systems?"
```

Both runtimes must share the same Rust parsing, interpretation, VFS and command implementations. The AI backend must remain optional and capability-limited. Ship a real engineering system, not a shell-shaped animation.

**BEGIN: inspect the repository, produce a brief PR-01 proposal, implement and verify PR-01, report, then stop.**
