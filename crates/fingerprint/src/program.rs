use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Program<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        for trivia in self.trivia.iter() {
            trivia.fingerprint_with_hasher(hasher, resolved_names, options);
        }

        for statement in self.statements.iter() {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Trivia<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self.kind {
            TriviaKind::WhiteSpace => {}
            TriviaKind::SingleLineComment | TriviaKind::MultiLineComment | TriviaKind::HashComment => {
                if options.is_important_comment(self.value) {
                    "comment".hash(hasher);
                    self.value.hash(hasher);
                }
            }
            TriviaKind::DocBlockComment => {
                if options.is_important_comment(self.value) {
                    "docblock".hash(hasher);
                    self.value.hash(hasher);
                }
            }
        }
    }
}
