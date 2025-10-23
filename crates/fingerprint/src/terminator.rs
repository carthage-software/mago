use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Terminator<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        "terminator".hash(hasher);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahash::AHasher;
    use mago_database::file::FileId;
    use mago_span::Position;
    use mago_span::Span;
    use mago_syntax::ast::ast::tag::ClosingTag;
    use mago_syntax::ast::ast::tag::OpeningTag;
    use mago_syntax::ast::ast::tag::ShortOpeningTag;
    use std::hash::Hasher;

    fn fingerprint_item<T: Fingerprintable>(item: &T, opts: &FingerprintOptions, names: &ResolvedNames) -> u64 {
        let mut hasher = AHasher::default();
        item.fingerprint_with_hasher(&mut hasher, names, opts);
        hasher.finish()
    }

    fn make_span(offset: u32, len: u32) -> Span {
        Span { file_id: FileId::zero(), start: Position { offset }, end: Position { offset: offset + len } }
    }

    #[test]
    fn test_all_terminators_equal() {
        let opts = FingerprintOptions::default();
        let names = ResolvedNames::default();

        let semicolon = Terminator::Semicolon(make_span(0, 1));
        let closing_tag = Terminator::ClosingTag(ClosingTag { span: make_span(0, 2) });
        let tag_pair = Terminator::TagPair(
            ClosingTag { span: make_span(0, 2) },
            OpeningTag::Short(ShortOpeningTag { span: make_span(2, 2) }),
        );

        assert_eq!(fingerprint_item(&semicolon, &opts, &names), fingerprint_item(&closing_tag, &opts, &names));
        assert_eq!(fingerprint_item(&semicolon, &opts, &names), fingerprint_item(&tag_pair, &opts, &names));
        assert_eq!(fingerprint_item(&closing_tag, &opts, &names), fingerprint_item(&tag_pair, &opts, &names));
    }

    #[test]
    fn test_terminator_consistent() {
        let opts = FingerprintOptions::default();
        let names = ResolvedNames::default();

        let term1 = Terminator::Semicolon(make_span(0, 1));
        let term2 = Terminator::Semicolon(make_span(100, 1));

        assert_eq!(fingerprint_item(&term1, &opts, &names), fingerprint_item(&term2, &opts, &names));
    }
}
