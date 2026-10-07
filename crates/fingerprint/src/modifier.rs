use crate::FingerprintOptions;
use crate::Fingerprintable;
use mago_names::ResolvedNames;
use mago_syntax::cst::Modifier;
use std::hash::Hash;

impl Fingerprintable for Modifier<'_> {
    #[inline]
    fn fingerprint_with_hasher<H>(
        &self,
        hasher: &mut H,
        _resolved_names: &ResolvedNames,
        _options: &FingerprintOptions<'_>,
    ) where
        H: std::hash::Hasher,
    {
        match self {
            Modifier::Static(_) => "static".hash(hasher),
            Modifier::Final(_) => "final".hash(hasher),
            Modifier::Abstract(_) => "abstract".hash(hasher),
            Modifier::Readonly(_) => "readonly".hash(hasher),
            Modifier::Public(_) => "public".hash(hasher),
            Modifier::PublicSet(_) => "public_set".hash(hasher),
            Modifier::Protected(_) => "protected".hash(hasher),
            Modifier::ProtectedSet(_) => "protected_set".hash(hasher),
            Modifier::Private(_) => "private".hash(hasher),
            Modifier::PrivateSet(_) => "private_set".hash(hasher),
        }
    }
}

#[inline]
pub fn fingerprint_modifiers<'modifier, H>(
    modifiers: impl IntoIterator<Item = &'modifier Modifier<'modifier>>,
    hasher: &mut H,
) where
    H: std::hash::Hasher,
{
    const NAMES: [&str; 9] = [
        "abstract",
        "final",
        "private",
        "private_set",
        "protected",
        "protected_set",
        "public_set",
        "readonly",
        "static",
    ];

    let mut indices = modifiers.into_iter().filter_map(|modifier| match modifier {
        Modifier::Public(_) => None,
        Modifier::Abstract(_) => Some(0),
        Modifier::Final(_) => Some(1),
        Modifier::Private(_) => Some(2),
        Modifier::PrivateSet(_) => Some(3),
        Modifier::Protected(_) => Some(4),
        Modifier::ProtectedSet(_) => Some(5),
        Modifier::PublicSet(_) => Some(6),
        Modifier::Readonly(_) => Some(7),
        Modifier::Static(_) => Some(8),
    });

    let Some(first) = indices.next() else {
        return;
    };

    let Some(second) = indices.next() else {
        NAMES[first].hash(hasher);
        return;
    };

    let mut counts = [0usize; NAMES.len()];
    counts[first] += 1;
    counts[second] += 1;
    for index in indices {
        counts[index] += 1;
    }

    for (name, count) in NAMES.into_iter().zip(counts) {
        for _ in 0..count {
            name.hash(hasher);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;

    use mago_span::Span;
    use mago_syntax::cst::Keyword;

    use super::*;

    #[derive(Default, Debug, PartialEq, Eq)]
    struct Writes(Vec<Vec<u8>>);

    impl Hasher for Writes {
        fn finish(&self) -> u64 {
            0
        }

        fn write(&mut self, bytes: &[u8]) {
            self.0.push(bytes.to_vec());
        }
    }

    fn sorted_writes(modifiers: &[&Modifier<'_>]) -> Writes {
        let mut names: Vec<_> = modifiers
            .iter()
            .filter_map(|modifier| match modifier {
                Modifier::Public(_) => None,
                Modifier::Static(_) => Some("static"),
                Modifier::Final(_) => Some("final"),
                Modifier::Abstract(_) => Some("abstract"),
                Modifier::Readonly(_) => Some("readonly"),
                Modifier::PublicSet(_) => Some("public_set"),
                Modifier::Protected(_) => Some("protected"),
                Modifier::ProtectedSet(_) => Some("protected_set"),
                Modifier::Private(_) => Some("private"),
                Modifier::PrivateSet(_) => Some("private_set"),
            })
            .collect();
        names.sort_unstable();
        let mut writes = Writes::default();
        for name in names {
            name.hash(&mut writes);
        }
        writes
    }

    #[test]
    fn modifier_counts_keep_sorted_hash_writes_and_duplicates() {
        let keyword = Keyword { span: Span::zero(), value: b"unused" };
        let kinds = [
            Modifier::Public(keyword),
            Modifier::Static(keyword),
            Modifier::Final(keyword),
            Modifier::Abstract(keyword),
            Modifier::Readonly(keyword),
            Modifier::PublicSet(keyword),
            Modifier::Protected(keyword),
            Modifier::ProtectedSet(keyword),
            Modifier::Private(keyword),
            Modifier::PrivateSet(keyword),
        ];

        let check = |modifiers: &[&Modifier<'_>]| {
            let mut writes = Writes::default();
            fingerprint_modifiers(modifiers.iter().copied(), &mut writes);
            assert_eq!(writes, sorted_writes(modifiers), "{modifiers:?}");
        };

        for length in 0..=4u32 {
            for mut value in 0..kinds.len().pow(length) {
                let mut modifiers = Vec::new();
                for _ in 0..length {
                    modifiers.push(&kinds[value % kinds.len()]);
                    value /= kinds.len();
                }
                check(&modifiers);
            }
        }

        let mut state = 0x56c1_6f48usize;
        for length in (0..=128).cycle().take(2048) {
            let modifiers: Vec<_> = std::iter::repeat_with(|| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                &kinds[state % kinds.len()]
            })
            .take(length)
            .collect();
            check(&modifiers);
        }
        for modifier in &kinds {
            check(&vec![modifier; 257]);
        }
    }
}
