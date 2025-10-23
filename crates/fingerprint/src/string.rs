use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::*;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for CompositeString<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        use CompositeString::*;

        match self {
            ShellExecute(s) => s.fingerprint_with_hasher(hasher, resolved_names, options),
            Interpolated(s) => s.fingerprint_with_hasher(hasher, resolved_names, options),
            Document(s) => s.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for ShellExecuteString<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "shell_execute".hash(hasher);
        for part in self.parts.iter() {
            part.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for InterpolatedString<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "interpolated_string".hash(hasher);
        for part in self.parts.iter() {
            part.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for DocumentString<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        match self.kind {
            DocumentKind::Heredoc => "heredoc".hash(hasher),
            DocumentKind::Nowdoc => "nowdoc".hash(hasher),
        };
        self.label.hash(hasher);
        self.indentation.fingerprint_with_hasher(hasher, resolved_names, options);
        for part in self.parts.iter() {
            part.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for DocumentIndentation {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        use DocumentIndentation::*;

        match self {
            None => "no_indent".hash(hasher),
            Whitespace(n) => {
                "whitespace".hash(hasher);
                n.hash(hasher);
            }
            Tab(n) => {
                "tab".hash(hasher);

                n.hash(hasher);
            }
            Mixed(spaces, tabs) => {
                "mixed".hash(hasher);

                spaces.hash(hasher);
                tabs.hash(hasher);
            }
        }
    }
}

impl Fingerprintable for StringPart<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        use StringPart::*;

        match self {
            Literal(l) => l.fingerprint_with_hasher(hasher, resolved_names, options),
            Expression(e) => {
                "string_expr".hash(hasher);
                e.fingerprint_with_hasher(hasher, resolved_names, options);
            }
            BracedExpression(b) => b.fingerprint_with_hasher(hasher, resolved_names, options),
        }
    }
}

impl Fingerprintable for LiteralStringPart<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) {
        "string_literal".hash(hasher);
        self.value.hash(hasher);
    }
}

impl Fingerprintable for BracedExpressionStringPart<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "string_braced_expr".hash(hasher);
        self.expression.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fingerprint_code;

    #[test]
    fn test_string_types_unique() {
        let shell = fingerprint_code("<?php `echo hello`;");
        let interpolated = fingerprint_code("<?php \"hello\";");
        let heredoc = fingerprint_code("<?php <<<EOT\nhello\nEOT;\n");
        let nowdoc = fingerprint_code("<?php <<<'EOT'\nhello\nEOT;\n");

        assert_ne!(shell, interpolated);
        assert_ne!(interpolated, heredoc);
        assert_ne!(heredoc, nowdoc);
    }

    #[test]
    fn test_string_interpolation() {
        let simple = fingerprint_code("<?php \"hello\";");
        let with_var = fingerprint_code("<?php \"hello $name\";");
        let with_braced = fingerprint_code("<?php \"hello {$name}\";");

        assert_ne!(simple, with_var);
        assert_ne!(simple, with_braced);
        assert_ne!(with_var, with_braced);
    }
}
