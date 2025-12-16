use serde::Serialize;
use serde::Serializer;
use serde::ser::SerializeStruct;

use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;

use crate::integration::Integration;
use crate::integration::IntegrationSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, Hash, PartialOrd)]
pub enum RuleRequirements {
    /// The rule is active if the specified PHP version range is met.
    PHPVersion(PHPVersionRange),
    /// The rule is active if the specified integration is enabled.
    Integration(Integration),
    /// The rule is active if **any** of the specified requirements are met (logical OR).
    Any(&'static [RuleRequirements]),
    /// The rule is active only if **all** of the specified requirements are met (logical AND).
    All(&'static [RuleRequirements]),
    /// The rule is always active, regardless of integrations.
    None,
}

impl RuleRequirements {
    /// Recursively finds all PHP version requirements in the tree.
    pub fn php_version_ranges(&self) -> Vec<PHPVersionRange> {
        match self {
            Self::PHPVersion(range) => vec![*range],
            Self::Any(requirements) | Self::All(requirements) => {
                requirements.iter().flat_map(RuleRequirements::php_version_ranges).collect()
            }
            _ => vec![],
        }
    }

    /// Extracts all required integration sets into a list representing a logical OR.
    ///
    /// This converts the requirement tree into Disjunctive Normal Form (DNF).
    ///
    /// The outer `Vec` represents an `OR`, and each inner `IntegrationSet` represents an `AND`.
    ///
    /// Example: `(A & B) | C` becomes `vec![{A, B}, {C}]`.
    #[must_use]
    pub fn required_integrations(&self) -> Vec<IntegrationSet> {
        self.dnf()
    }

    /// Checks if a given set of enabled integrations satisfies the rule's requirements.
    #[must_use]
    pub fn are_met_by(&self, configured_php_version: PHPVersion, configured_integrations: IntegrationSet) -> bool {
        match self {
            Self::None => true,
            Self::PHPVersion(range) => range.includes(configured_php_version),
            Self::Integration(integration) => configured_integrations.contains(*integration),
            Self::Any(requirements) => {
                let mut set = IntegrationSet::empty();
                for req in *requirements {
                    if let Self::Integration(i) = req {
                        set.insert(*i);
                    } else {
                        return requirements
                            .iter()
                            .any(|req| req.are_met_by(configured_php_version, configured_integrations));
                    }
                }

                configured_integrations.intersects(set)
            }
            Self::All(requirements) => {
                let mut set = IntegrationSet::empty();
                for req in *requirements {
                    if let Self::Integration(i) = req {
                        set.insert(*i);
                    } else {
                        return requirements
                            .iter()
                            .all(|req| req.are_met_by(configured_php_version, configured_integrations));
                    }
                }

                configured_integrations.is_superset_of(set)
            }
        }
    }

    fn dnf(&self) -> Vec<IntegrationSet> {
        match self {
            Self::Integration(i) => vec![IntegrationSet::only(*i)],
            Self::Any(reqs) => reqs.iter().flat_map(RuleRequirements::dnf).collect(),
            Self::All(reqs) => {
                let mut dnf = vec![IntegrationSet::empty()];

                for req in *reqs {
                    let req_dnf = req.dnf();
                    if req_dnf.is_empty() {
                        return vec![];
                    }

                    let mut next_dnf = Vec::new();
                    for existing_set in &dnf {
                        for new_set in &req_dnf {
                            next_dnf.push(existing_set.union(*new_set));
                        }
                    }

                    dnf = next_dnf;
                }

                dnf
            }
            Self::PHPVersion(_) | Self::None => vec![],
        }
    }
}

impl Serialize for RuleRequirements {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("RuleRequirements", 2)?;
        state.serialize_field("php-versions", &self.php_version_ranges())?;
        state.serialize_field("integrations", &self.required_integrations())?;
        state.end()
    }
}
