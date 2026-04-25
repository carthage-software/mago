//! Lexer fuzz target.
//!
//! Drives the lexer on arbitrary bytes until EOF and asserts losslessness:
//! when the lexer succeeds end-to-end, concatenating every token's `value`
//! must reproduce the original input byte-for-byte. The lexer is contractually
//! lossless and the formatter / printer / source-span machinery all rely on
//! that, so a mismatch here is a real bug, not just an "interesting" output.
//!
//! Lex errors are still expected on malformed input and are silently dropped
//! (we can't sensibly reconstruct from a partial stream).
//!
//! Crashes - panics, aborts, stack overflows - are bugs unconditionally.

#![no_main]

use libfuzzer_sys::fuzz_target;

use mago_database::file::FileId;
use mago_syntax::lexer::Lexer;
use mago_syntax::settings::LexerSettings;
use mago_syntax_core::input::Input;

fuzz_target!(|data: &[u8]| {
    if std::str::from_utf8(data).is_err() {
        return;
    }

    let input = Input::new(FileId::zero(), data);
    let mut lexer = Lexer::new(input, LexerSettings::default());

    let mut reconstructed: Vec<u8> = Vec::with_capacity(data.len());
    let mut hit_error = false;

    while let Some(result) = lexer.advance() {
        match result {
            Ok(token) => reconstructed.extend_from_slice(token.value.as_bytes()),
            Err(_) => {
                hit_error = true;
                break;
            }
        }
    }

    if !hit_error {
        assert_eq!(
            data,
            reconstructed.as_slice(),
            "lexer is not lossless: concatenated token values do not reproduce the input",
        );
    }
});
