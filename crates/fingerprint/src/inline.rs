use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;
use std::hash::Hash;

impl Fingerprintable for Inline<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        "inline".hash(hasher);
        self.kind.hash(hasher);
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;

    use ahash::AHasher;

    use mago_database::file::FileId;
    use mago_span::Position;
    use mago_span::Span;
    use mago_syntax::ast::ast::inline::InlineKind;

    use super::*;

    fn make_inline(value: &str, kind: InlineKind) -> Inline<'_> {
        Inline {
            kind,
            value,
            span: Span {
                file_id: FileId::zero(),
                start: Position { offset: 0 },
                end: Position { offset: value.len() as u32 },
            },
        }
    }

    fn fingerprint_inline(inline: &Inline, opts: &FingerprintOptions, names: &ResolvedNames) -> u64 {
        let mut hasher = AHasher::default();
        inline.fingerprint_with_hasher(&mut hasher, names, opts);
        hasher.finish()
    }

    #[test]
    fn test_inline_content_excluded() {
        let opts = FingerprintOptions::default();
        let names = ResolvedNames::default();

        let inline1 = make_inline("Hello", InlineKind::Text);
        let inline2 = make_inline("World", InlineKind::Text);

        assert_eq!(fingerprint_inline(&inline1, &opts, &names), fingerprint_inline(&inline2, &opts, &names));
    }

    #[test]
    fn test_inline_kind_included() {
        let opts = FingerprintOptions::default();
        let names = ResolvedNames::default();

        let inline_text = make_inline("Some text", InlineKind::Text);
        let inline_html = make_inline("Some text", InlineKind::Shebang);

        assert_ne!(fingerprint_inline(&inline_text, &opts, &names), fingerprint_inline(&inline_html, &opts, &names));
    }
}
