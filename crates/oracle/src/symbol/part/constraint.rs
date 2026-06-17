#[cfg(feature = "serde")]
use serde::Serialize;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct SymbolConstraint<'arena> {
    pub ranges: &'arena [PHPVersionRange],
}

impl SymbolConstraint<'_> {
    #[inline]
    #[must_use]
    pub const fn unconstrained() -> Self {
        Self { ranges: &[] }
    }

    #[inline]
    #[must_use]
    pub const fn is_unconstrained(&self) -> bool {
        self.ranges.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn allows_version(&self, version: PHPVersion) -> bool {
        if self.ranges.is_empty() {
            return true;
        }

        self.ranges.iter().any(|r| r.includes(version))
    }

    #[inline]
    #[must_use]
    pub fn allows_version_range(&self, range: PHPVersionRange) -> bool {
        if self.ranges.is_empty() {
            return true;
        }

        let min = range.min.unwrap_or(PHPVersion::from_version_id(0));
        let max = range.max.unwrap_or(PHPVersion::from_version_id(u32::MAX));

        if min > max {
            return true;
        }

        let mut sorted: Vec<&PHPVersionRange> = self.ranges.iter().collect();
        sorted.sort_by_key(|r| r.min);

        let mut next_required = min;
        for r in sorted {
            let r_min = r.min.unwrap_or(PHPVersion::from_version_id(0));
            let r_max = r.max.unwrap_or(PHPVersion::from_version_id(u32::MAX));

            if r_max < next_required {
                continue;
            }

            if r_min > next_required {
                return false;
            }

            if r_max >= max {
                return true;
            }

            next_required = PHPVersion::from_version_id(r_max.to_version_id().saturating_add(1));
        }

        false
    }
}
