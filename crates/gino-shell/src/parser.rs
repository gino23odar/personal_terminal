//! Single-command parsing only. See docs/SHELL_GRAMMAR.md for compatibility rules.
use std::{fmt, ops::Range};

/// Bound synchronous parsing and allocations before inspecting input.
pub const MAX_INPUT_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordPartKind {
    Unquoted,
    SingleQuoted,
    DoubleQuoted,
    Escaped,
    DoubleQuotedEscape,
}

/// Decoded text plus its quoting provenance; no expansion has taken place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WordPart {
    pub kind: WordPartKind,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    /// Half-open UTF-8 byte range, including source quotes and backslashes.
    pub span: Range<usize>,
    pub parts: Vec<WordPart>,
}

impl Word {
    /// Concatenate decoded fragments, preserving empty arguments.
    pub fn value(&self) -> String {
        self.parts.iter().map(|part| part.text.as_str()).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleCommand {
    pub name: Word,
    pub arguments: Vec<Word>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    InputTooLong,
    InvalidControl(char),
    UnsupportedSyntax(&'static str),
    UnclosedQuote(char),
    TrailingEscape,
    EmptyCommandName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    /// Half-open UTF-8 byte range. No source text is embedded in diagnostics.
    pub span: Range<usize>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "parse error at bytes {}..{}: ",
            self.span.start, self.span.end
        )?;
        match &self.kind {
            ParseErrorKind::InputTooLong => write!(f, "input exceeds {MAX_INPUT_BYTES} bytes"),
            ParseErrorKind::InvalidControl(c) => {
                write!(f, "control character {c:?} is not allowed")
            }
            ParseErrorKind::UnsupportedSyntax(op) => {
                write!(f, "syntax {op:?} is not supported yet")
            }
            ParseErrorKind::UnclosedQuote(c) => write!(f, "unclosed {c:?} quote"),
            ParseErrorKind::TrailingEscape => {
                f.write_str("backslash requires a following character")
            }
            ParseErrorKind::EmptyCommandName => f.write_str("command name cannot be empty"),
        }
    }
}
impl std::error::Error for ParseError {}

/// Produce decoded words or an error for the entire input, never a partial command.
/// Operators are recognized and rejected here until composition PRs introduce them.
pub fn tokenize(input: &str) -> Result<Vec<Word>, ParseError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(ParseError {
            kind: ParseErrorKind::InputTooLong,
            span: 0..input.len(),
        });
    }
    // Reject multiline/control input even inside quotes or after an escape.
    for (offset, c) in input.char_indices() {
        let kind = match c {
            '\r' | '\n' => Some(ParseErrorKind::UnsupportedSyntax("newline")),
            c if c.is_control() && c != '\t' => Some(ParseErrorKind::InvalidControl(c)),
            _ => None,
        };
        if let Some(kind) = kind {
            return Err(ParseError {
                kind,
                span: offset..offset + c.len_utf8(),
            });
        }
    }
    Lexer { input, offset: 0 }.words()
}

/// Blank input is a no-op. Unknown command names are syntactically valid;
/// registry lookup and execution are the interpreter's responsibility in PR-05.
pub fn parse(input: &str) -> Result<Option<SimpleCommand>, ParseError> {
    let mut words = tokenize(input)?.into_iter();
    let Some(name) = words.next() else {
        return Ok(None);
    };
    if name.value().is_empty() {
        return Err(ParseError {
            kind: ParseErrorKind::EmptyCommandName,
            span: name.span,
        });
    }
    Ok(Some(SimpleCommand {
        name,
        arguments: words.collect(),
    }))
}

struct Lexer<'a> {
    input: &'a str,
    offset: usize,
}

impl Lexer<'_> {
    fn peek(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }
    fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.offset += c.len_utf8();
        Some(c)
    }
    fn error(&self, kind: ParseErrorKind, start: usize, length: usize) -> ParseError {
        ParseError {
            kind,
            span: start..start + length,
        }
    }
    fn reject_syntax(&self, quoted: bool) -> Result<(), ParseError> {
        let rest = &self.input[self.offset..];
        let operators: &[&'static str] = if quoted {
            &["$(", "${", "`"]
        } else {
            &[
                "<<<", "<<", ">>", "&&", "||", ";;", "$(", "${", "|", "&", ";", "<", ">", "(", ")",
                "`",
            ]
        };
        if let Some(op) = operators.iter().find(|op| rest.starts_with(**op)) {
            return Err(self.error(ParseErrorKind::UnsupportedSyntax(op), self.offset, op.len()));
        }
        Ok(())
    }
    fn words(&mut self) -> Result<Vec<Word>, ParseError> {
        let mut words = Vec::new();
        while self.peek().is_some() {
            if matches!(self.peek(), Some(' ' | '\t')) {
                self.advance();
                continue;
            }
            let start = self.offset;
            let mut parts = Vec::new();
            while let Some(c) = self.peek() {
                if matches!(c, ' ' | '\t') {
                    break;
                }
                match c {
                    '\'' | '"' => self.quoted(&mut parts, c)?,
                    '\\' => self.escape(&mut parts, false)?,
                    _ => {
                        self.reject_syntax(false)?;
                        self.advance();
                        append(&mut parts, WordPartKind::Unquoted, c);
                    }
                }
            }
            words.push(Word {
                span: start..self.offset,
                parts,
            });
        }
        Ok(words)
    }
    fn quoted(&mut self, parts: &mut Vec<WordPart>, quote: char) -> Result<(), ParseError> {
        let start = self.offset;
        self.advance();
        let kind = if quote == '\'' {
            WordPartKind::SingleQuoted
        } else {
            WordPartKind::DoubleQuoted
        };
        // Keep even empty quoted fragments, so '' is an argument, not missing input.
        parts.push(WordPart {
            kind,
            text: String::new(),
        });
        while let Some(c) = self.peek() {
            if c == quote {
                self.advance();
                return Ok(());
            }
            if quote == '"' {
                if c == '\\' {
                    self.escape(parts, true)?;
                    continue;
                }
                self.reject_syntax(true)?;
            }
            self.advance();
            append(parts, kind, c);
        }
        Err(self.error(ParseErrorKind::UnclosedQuote(quote), start, 1))
    }
    fn escape(&mut self, parts: &mut Vec<WordPart>, quoted: bool) -> Result<(), ParseError> {
        let start = self.offset;
        self.advance();
        let Some(c) = self.peek() else {
            return Err(self.error(ParseErrorKind::TrailingEscape, start, 1));
        };
        if quoted && !matches!(c, '"' | '\\' | '$' | '`') {
            // In double quotes, a backslash before any other character is literal.
            append(parts, WordPartKind::DoubleQuoted, '\\');
        } else {
            self.advance();
            append(
                parts,
                if quoted {
                    WordPartKind::DoubleQuotedEscape
                } else {
                    WordPartKind::Escaped
                },
                c,
            );
        }
        Ok(())
    }
}

fn append(parts: &mut Vec<WordPart>, kind: WordPartKind, c: char) {
    if let Some(last) = parts.last_mut().filter(|part| part.kind == kind) {
        last.text.push(c);
    } else {
        parts.push(WordPart {
            kind,
            text: c.to_string(),
        });
    }
}
