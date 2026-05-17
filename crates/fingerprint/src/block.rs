use mago_names::ResolvedNames;
use mago_syntax::ast::Block;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Block<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        for statement in &self.statements {
            statement.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}
