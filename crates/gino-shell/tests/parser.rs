use gino_shell::{MAX_INPUT_BYTES, ParseErrorKind as Error, WordPartKind as Kind, parse, tokenize};

fn values(input: &str) -> Vec<String> {
    tokenize(input).unwrap().iter().map(|w| w.value()).collect()
}

#[test]
fn blank_input_is_a_noop_and_unknown_names_are_syntax_valid() {
    for input in ["", " ", "\t  \t"] {
        assert_eq!(parse(input).unwrap(), None);
    }
    let command = parse("unknown-command --flag value").unwrap().unwrap();
    assert_eq!(command.name.value(), "unknown-command");
    assert_eq!(
        command
            .arguments
            .iter()
            .map(|w| w.value())
            .collect::<Vec<_>>(),
        ["--flag", "value"]
    );
}

#[test]
fn quotes_preserve_spaces_empty_arguments_and_adjacent_fragments() {
    assert_eq!(
        values(r#"echo 'two words' "three words" '' "" pre"mid dle"'post'"#),
        ["echo", "two words", "three words", "", "", "premid dlepost"]
    );
    assert_eq!(values("echo 'a\tb'"), ["echo", "a\tb"]);
    assert_eq!(values(r#"echo 'a"b\c' "a'b""#), ["echo", "a\"b\\c", "a'b"]);
}

#[test]
fn escapes_follow_the_documented_quote_rules() {
    assert_eq!(
        values(r#"echo a\ b \| \" \' \\ \é"#),
        ["echo", "a b", "|", "\"", "'", "\\", "é"]
    );
    assert_eq!(
        values(r#"echo "a\"b\\c\$d\`e\n\ q""#),
        ["echo", "a\"b\\c$d`e\\n\\ q"]
    );
}

#[test]
fn expansion_is_deferred_and_quote_provenance_is_retained() {
    let words = tokenize(r#"$X "$X" '$X' \$X "\$X" ~ *.rs #tag A=B"#).unwrap();
    assert_eq!(
        words.iter().map(|w| w.value()).collect::<Vec<_>>(),
        ["$X", "$X", "$X", "$X", "$X", "~", "*.rs", "#tag", "A=B"]
    );
    assert_eq!(words[0].parts[0].kind, Kind::Unquoted);
    assert_eq!(words[1].parts[0].kind, Kind::DoubleQuoted);
    assert_eq!(words[2].parts[0].kind, Kind::SingleQuoted);
    assert_eq!(words[3].parts[0].kind, Kind::Escaped);
    assert_eq!(words[4].parts[1].kind, Kind::DoubleQuotedEscape);
}

#[test]
fn spans_are_utf8_byte_ranges_and_unicode_whitespace_is_literal() {
    let input = "  écho\t\"🦀 x\" a\u{a0}b";
    let words = tokenize(input).unwrap();
    assert_eq!(words[0].span, 2..7);
    assert_eq!(words[1].span, 8..16);
    assert_eq!(&input[words[1].span.clone()], "\"🦀 x\"");
    assert_eq!(words[2].value(), "a\u{a0}b");
}

#[test]
fn empty_command_names_are_rejected_but_empty_arguments_are_valid() {
    for input in ["''", "\"\" arg", "''\"\""] {
        let error = parse(input).unwrap_err();
        assert_eq!(error.kind, Error::EmptyCommandName);
        assert_eq!(error.span.start, 0);
    }
    assert_eq!(parse("echo ''").unwrap().unwrap().arguments[0].value(), "");
    assert_eq!(values("''"), [""]);
}

#[test]
fn malformed_quotes_and_escapes_report_the_source_location() {
    for quote in ['\'', '"'] {
        let error = parse(&format!("écho {quote}unfinished")).unwrap_err();
        assert_eq!(error.kind, Error::UnclosedQuote(quote));
        assert_eq!(error.span, 6..7);
    }
    for input in ["echo \\", "echo \"\\"] {
        let error = parse(input).unwrap_err();
        assert_eq!(error.kind, Error::TrailingEscape);
        assert_eq!(error.span, input.len() - 1..input.len());
    }
}

#[test]
fn unsupported_operators_are_rejected_without_partial_commands() {
    for op in [
        "<<<", "<<", ">>", "&&", "||", ";;", "$(", "${", "|", "&", ";", "<", ">", "(", ")", "`",
    ] {
        for prefix in ["", "echo ok", "echo é"] {
            let input = format!("{prefix}{op}tail");
            let error = parse(&input).unwrap_err();
            assert_eq!(error.kind, Error::UnsupportedSyntax(op), "{input}");
            assert_eq!(error.span, prefix.len()..prefix.len() + op.len());
        }
    }
}

#[test]
fn quoting_or_escaping_allows_literal_operators() {
    assert_eq!(
        values(r#"echo '|&;<>()`$( ${' "|&;<>()" \|\&\;\<\>\(\)\`"#),
        ["echo", "|&;<>()`$( ${", "|&;<>()", "|&;<>()`"]
    );
    assert_eq!(
        values(r#"echo "\$(x) \${x} \`x\`""#),
        ["echo", "$(x) ${x} `x`"]
    );
    for op in ["$(", "${", "`"] {
        let error = parse(&format!("echo \"{op}x\"")).unwrap_err();
        assert_eq!(error.kind, Error::UnsupportedSyntax(op));
        assert_eq!(error.span, 6..6 + op.len());
    }
}

#[test]
fn control_and_multiline_input_is_rejected_even_when_quoted_or_escaped() {
    for c in ['\n', '\r', '\0', '\u{1b}', '\u{7f}', '\u{85}'] {
        for prefix in ["é", "'é", "\"é", "é\\"] {
            let error = parse(&format!("{prefix}{c}")).unwrap_err();
            let kind = if matches!(c, '\n' | '\r') {
                Error::UnsupportedSyntax("newline")
            } else {
                Error::InvalidControl(c)
            };
            assert_eq!(error.kind, kind);
            assert_eq!(error.span, prefix.len()..prefix.len() + c.len_utf8());
            assert!(!error.to_string().contains(c));
        }
    }
    assert_eq!(
        parse("|\n").unwrap_err().kind,
        Error::UnsupportedSyntax("newline")
    );
}

#[test]
fn input_limit_is_in_bytes_and_checked_before_syntax() {
    assert!(parse(&"é".repeat(MAX_INPUT_BYTES / 2)).is_ok());
    let input = format!("|{}", "é".repeat(MAX_INPUT_BYTES / 2));
    let error = parse(&input).unwrap_err();
    assert_eq!(error.kind, Error::InputTooLong);
    assert_eq!(error.span, 0..input.len());
}

#[test]
fn short_input_corpus_has_valid_spans_and_never_panics() {
    let alphabet = [
        "a", "é", "🦀", " ", "\t", "'", "\"", "\\", "$", "(", "|", "\n",
    ];
    for a in alphabet {
        for b in alphabet {
            for c in alphabet {
                let input = format!("{a}{b}{c}");
                match tokenize(&input) {
                    Ok(words) => {
                        for word in words {
                            assert!(input.get(word.span.clone()).is_some());
                            assert!(word.span.start < word.span.end);
                        }
                    }
                    Err(error) => assert!(input.get(error.span).is_some()),
                }
            }
        }
    }
}
