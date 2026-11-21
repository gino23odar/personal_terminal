# Shell grammar — PR-04

The portable Rust core exposes `tokenize(&str)` and `parse(&str)`. These functions produce syntax data only: they do not access the registry, execute commands, expand variables, mutate sessions or invoke a host shell. CLI and browser integration remain planned.

## Single-command syntax

Input contains zero or one command. ASCII space and tab separate words; leading and trailing separators are ignored. Other Unicode whitespace is literal, except control characters rejected below. A word combines adjacent unquoted, single-quoted, double-quoted and escaped fragments. Quotes are removed from decoded values; empty quoted arguments are retained.

- Single quotes preserve everything between matching single quotes literally. Backslash has no special meaning there.
- Outside quotes, backslash consumes the following character as literal text.
- In double quotes, backslash escapes only double quote, backslash, dollar sign and backtick. Before any other character, the backslash is retained.
- There are no C-style escapes: `\n` outside quotes decodes to `n`, and inside double quotes remains backslash plus `n`.
- `$NAME`, `~`, glob patterns, `#` and assignment-shaped words are literal text. Expansion, comments and assignment syntax are not implemented.

| Input                         | Decoded words / result                       |
| ----------------------------- | -------------------------------------------- |
| `echo hello`                  | `echo`, `hello`                              |
| `echo 'two words' ""`         | `echo`, `two words`, empty argument          |
| `echo pre"mid dle"'post'`     | `echo`, `premid dlepost`                     |
| `echo a\ b`                   | `echo`, `a b`                                |
| `echo "$HOME"`                | `echo`, literal `$HOME`                      |
| `echo 'a;b'`                  | `echo`, `a;b`                                |
| `echo a;echo b`               | Unsupported syntax error; no partial command |
| blank or space/tab-only input | `parse` returns `Ok(None)`                   |

`parse` returns `SimpleCommand { name, arguments }`. Names need not exist in the registry or match its registration rules; lookup errors belong to the future interpreter. A decoded empty command name is rejected. `tokenize` can return an empty first word because it has no command-role validation.

Each `Word` owns a half-open UTF-8 byte `span` covering its original source, including quotes and backslashes, and decoded `parts`. `Word::value()` joins their text. `WordPartKind` distinguishes unquoted, single-quoted, double-quoted, outside escape and double-quoted escape provenance for future expansion rules. Parts are fragments, not a concrete syntax tree; individual parts have no source spans. No expansion happens now.

## Unsupported syntax and errors

Unquoted pipes, redirects, chaining, backgrounding, grouping and substitution are rejected: `|`, `||`, `&`, `&&`, `;`, `;;`, `<`, `<<`, `<<<`, `>`, `>>`, `(`, `)`, backtick, `$(` and `${`. Listed multi-character operators are matched longest first. Ordinary operators can appear literally when quoted or escaped. Backtick, `$(` and `${` are also rejected in double quotes; single quoting or escaping the triggering character makes them literal.

`ParseError` carries a typed `ParseErrorKind` and a UTF-8 byte range. Unsupported syntax points at the operator, unclosed quotes at the opening quote, and trailing escapes at the backslash. A trailing escape inside an unclosed double quote reports the escape first. Empty command names point at the first word. Diagnostics do not echo input and escape control characters.

The parser accepts at most 65,536 UTF-8 bytes, checked before scanning. Oversized input reports the whole input range. A preflight scan then rejects CR/LF anywhere, even inside quotes or after backslash, and all Rust `char::is_control()` characters except tab. These validation errors take priority over lexical errors. Adapters must pass a single logical line without its line terminator; multiline input and continuation are unsupported. Remaining lexical errors are reported left to right. All failures return an error for the entire input, never a partially parsed command.

The implementation scans iteratively without recursion, external dependencies or platform I/O. Its synchronous work and allocations are bounded by the input limit. Parsing has no cancellation hook; asynchronous execution and cancellation remain separate concerns.

## Verification and next integration

`cargo test -p gino-shell --test parser --locked` covers valid/invalid syntax, empty words, quote provenance, operator rejection, UTF-8 spans, control input, byte limits and a deterministic 1,728-input short corpus. This is regression coverage, not exhaustive fuzzing. The existing CI also compiles the parser and its tests for WASM; it does not execute them in a browser.

PR-05 consumes this AST, resolves names through `CommandRegistry`, and executes built-ins through the context/I/O/session contracts. It handles blank input and syntax failures before dispatch; see [execution rules](COMMANDS.md). Pipes, redirection and expansion remain at their roadmap PRs. Interactive CLI and browser behavior are not delivered by PR-04.
