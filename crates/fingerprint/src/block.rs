use mago_names::ResolvedNames;
use mago_syntax::ast::Block;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Block<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
