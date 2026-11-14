#![allow(clippy::unwrap_used)]

use mago_database::file::FileId;
use mago_syntax_core::input::Input;
use mago_twig_syntax::lexer::TwigLexer;
use mago_twig_syntax::settings::LexerSettings;
use mago_twig_syntax::token::TwigTokenKind;

pub fn assert_preserves_syntax(source: &[u8], formatted: &[u8]) {
    assert_eq!(tokens(source), tokens(formatted), "Formatting changed tokens or template text");
}

fn tokens(source: &[u8]) -> Vec<(TwigTokenKind, Vec<u8>)> {
    let mut lexer = TwigLexer::new(Input::new(FileId::zero(), source), LexerSettings);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.advance() {
        let token = token.unwrap();
        match token.kind {
            TwigTokenKind::Whitespace => {}
            TwigTokenKind::QuestionColon => {
                tokens.push((TwigTokenKind::Question, b"?".to_vec()));
                tokens.push((TwigTokenKind::Colon, b":".to_vec()));
            }
            TwigTokenKind::Comment => {
                let value = token.value;
                let start = 2 + usize::from(value.get(2).is_some_and(|byte| matches!(byte, b'-' | b'~')));
                let end = value.len()
                    - 2
                    - usize::from(
                        value.get(value.len().saturating_sub(3)).is_some_and(|byte| matches!(byte, b'-' | b'~')),
                    );
                if start <= end {
                    for part in [&value[..start], value[start..end].trim_ascii(), &value[end..]] {
                        tokens.push((token.kind, part.to_vec()));
                    }
                } else {
                    tokens.push((token.kind, value.to_vec()));
                }
            }
            TwigTokenKind::NotIn
            | TwigTokenKind::StartsWith
            | TwigTokenKind::EndsWith
            | TwigTokenKind::HasSome
            | TwigTokenKind::HasEvery
            | TwigTokenKind::SameAs
            | TwigTokenKind::DivisibleBy => {
                let mut value = Vec::new();
                for word in token.value.split(u8::is_ascii_whitespace).filter(|word| !word.is_empty()) {
                    if !value.is_empty() {
                        value.push(b' ');
                    }
                    value.extend_from_slice(word);
                }
                tokens.push((token.kind, value));
            }
            _ => tokens.push((token.kind, token.value.to_vec())),
        }
    }

    tokens
}
