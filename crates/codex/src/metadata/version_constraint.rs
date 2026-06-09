use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;

/// Tracks the PHP version intervals in which a symbol is available, derived
/// from `Mago\AvailableSince` / `Mago\AvailableUntil` attributes during
/// scanning.
///
/// Both attributes are repeatable, so a symbol can declare disjoint
/// availability ranges (for example "available 8.1–8.3, removed in 8.4,
/// brought back in 8.5"). An empty range list means "always available";
/// otherwise a version is allowed when *some* range contains it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VersionConstraint {
    /// Disjoint availability intervals in source order. Empty = unconstrained.
    pub ranges: Vec<PHPVersionRange>,
}

impl VersionConstraint {
    #[inline]
    #[must_use]
    pub const fn unconstrained() -> Self {
        Self { ranges: Vec::new() }
    }

    #[inline]
    #[must_use]
    pub const fn is_unconstrained(&self) -> bool {
        self.ranges.is_empty()
    }

    /// Records a new `Mago\AvailableSince(version)` claim — opens a fresh
    /// range that stays open on the right until a matching `AvailableUntil`
    /// claim closes it.
    #[inline]
    pub fn push_since(&mut self, version: PHPVersion) {
        self.ranges.push(PHPVersionRange::from(version));
    }

    /// Records a new `Mago\AvailableUntil(version)` claim. Closes the most
    /// recent open range if there is one; otherwise opens a brand-new range
    /// with no `min` bound.
    #[inline]
    pub fn push_until(&mut self, version: PHPVersion) {
        if let Some(last) = self.ranges.last_mut()
            && last.max.is_none()
        {
            last.max = Some(version);
            return;
        }

        self.ranges.push(PHPVersionRange::until(version));
    }

    /// Folds `other` into `self` by unioning their availability ranges.
    #[inline]
    pub fn merge(&mut self, other: VersionConstraint) {
        if self.is_unconstrained() || other.is_unconstrained() {
            self.ranges.clear();
            return;
        }

        self.ranges.extend(other.ranges);
    }

    /// Returns `true` when `version` is allowed by *any* range in this
    /// constraint.
    #[inline]
    #[must_use]
    pub fn allows_version(&self, version: PHPVersion) -> bool {
        if self.ranges.is_empty() {
            return true;
        }

        self.ranges.iter().any(|r| r.includes(version))
    }

    /// Returns `true` when *every* PHP version in `range` is covered by the
    /// union of ranges in this constraint. An open `min`/`max` on `range` is
    /// treated as the platform's known low/high bound.
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

            // Skip ranges that end before what we still need to cover.
            if r_max < next_required {
                continue;
            }

            // If this range opens after the next-required version, there's a
            // gap that no later range can close (ranges are sorted ascending
            // by `min`).
            if r_min > next_required {
                return false;
            }

            if r_max >= max {
                return true;
            }

            // LocalArena past this range. There's no `+1` notion on PHPVersion, so
            // re-cast through the packed id; every range we'd be looking for
            // next must start strictly after `r_max`.
            next_required = PHPVersion::from_version_id(r_max.to_version_id().saturating_add(1));
        }

        false
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn v(major: u32, minor: u32, patch: u32) -> PHPVersion {
        PHPVersion::new(major, minor, patch)
    }

    #[test]
    fn unconstrained_allows_everything() {
        let c = VersionConstraint::unconstrained();
        assert!(c.allows_version(v(7, 0, 0)));
        assert!(c.allows_version(v(8, 5, 0)));
        assert!(c.allows_version_range(PHPVersionRange::between(v(7, 0, 0), v(9, 0, 0))));
    }

    #[test]
    fn since_only_open_on_the_right() {
        let mut c = VersionConstraint::unconstrained();
        c.push_since(v(8, 1, 0));
        assert!(!c.allows_version(v(8, 0, 0)));
        assert!(c.allows_version(v(8, 1, 0)));
        assert!(c.allows_version(v(8, 5, 0)));
    }

    #[test]
    fn until_only_open_on_the_left() {
        let mut c = VersionConstraint::unconstrained();
        c.push_until(v(8, 3, 0));
        assert!(c.allows_version(v(7, 0, 0)));
        assert!(c.allows_version(v(8, 3, 0)));
        assert!(!c.allows_version(v(8, 4, 0)));
    }

    #[test]
    fn since_then_until_closes_the_range() {
        let mut c = VersionConstraint::unconstrained();
        c.push_since(v(8, 1, 0));
        c.push_until(v(8, 3, 0));
        assert!(!c.allows_version(v(8, 0, 0)));
        assert!(c.allows_version(v(8, 1, 0)));
        assert!(c.allows_version(v(8, 3, 0)));
        assert!(!c.allows_version(v(8, 4, 0)));
    }

    #[test]
    fn disjoint_ranges_compose() {
        let mut c = VersionConstraint::unconstrained();
        c.push_since(v(8, 1, 0));
        c.push_until(v(8, 3, 0));
        c.push_since(v(8, 5, 0));

        assert!(!c.allows_version(v(8, 0, 0)));
        assert!(c.allows_version(v(8, 1, 0)));
        assert!(c.allows_version(v(8, 3, 0)));
        assert!(!c.allows_version(v(8, 4, 0)));
        assert!(c.allows_version(v(8, 5, 0)));
        assert!(c.allows_version(v(9, 0, 0)));
    }

    #[test]
    fn range_query_requires_full_coverage() {
        let mut c = VersionConstraint::unconstrained();
        c.push_since(v(8, 1, 0));
        c.push_until(v(8, 3, 0));
        c.push_since(v(8, 5, 0));

        // 8.0–8.7 has a gap at 8.4, so this fails.
        assert!(!c.allows_version_range(PHPVersionRange::between(v(8, 0, 0), v(8, 7, 0))));
        // 8.5+ is fully inside the open right range.
        assert!(c.allows_version_range(PHPVersionRange::between(v(8, 5, 0), v(8, 7, 0))));
        // 8.1–8.3 sits inside the first range.
        assert!(c.allows_version_range(PHPVersionRange::between(v(8, 1, 0), v(8, 3, 0))));
    }
}
