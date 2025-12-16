use std::hash::Hash;

use mago_names::ResolvedNames;
use mago_syntax::ast::Constant;
use mago_syntax::ast::ConstantItem;

use crate::FingerprintOptions;
use crate::Fingerprintable;

impl Fingerprintable for Constant<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "const_stmt".hash(hasher);
        for attr_list in &self.attribute_lists {
            attr_list.fingerprint_with_hasher(hasher, resolved_names, options);
        }
        for item in &self.items {
            item.fingerprint_with_hasher(hasher, resolved_names, options);
        }
    }
}

impl Fingerprintable for ConstantItem<'_> {
    fn fingerprint_with_hasher<H: std::hash::Hasher>(
        &self,
        hasher: &mut H,
        resolved_names: &ResolvedNames,
        options: &FingerprintOptions<'_>,
    ) {
        "const_item".hash(hasher);
        self.name.fingerprint_with_hasher(hasher, resolved_names, options);
        self.value.fingerprint_with_hasher(hasher, resolved_names, options);
    }
}
