//! Twig lexer fuzz target.

#![no_main]

use libfuzzer_sys::fuzz_target;

use mago_database::file::FileId;
use mago_syntax_core::input::Input;
use mago_twig_syntax::lexer::TwigLexer;
use mago_twig_syntax::settings::LexerSettings;

fuzz_target!(|data: &[u8]| {
    if std::str::from_utf8(data).is_err() {
        return;
    }

    let input = Input::new(FileId::zero(), data);
    let mut lexer = TwigLexer::new(input, LexerSettings::default());

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
            "twig lexer is not lossless: concatenated token values do not reproduce the input",
        );
    }
});
