use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::cst::Program;
use mago_syntax::cst::Trivia;
use mago_syntax::cst::TriviaKind;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Program<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        for trivia in &self.trivia {
            trivia.fingerprint_with_hasher(hasher, resolved_names, options);
        }

        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for Trivia<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
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
