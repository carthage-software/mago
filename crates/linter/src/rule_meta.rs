use crate::category::Category;
use crate::requirements::RuleRequirements;

#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RuleMeta {
    pub name: &'static str,
    pub code: &'static str,
    pub description: &'static str,
    pub good_example: &'static str,
    pub bad_example: &'static str,
    pub category: Category,
    pub requirements: RuleRequirements,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct RuleEntry {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub meta: &'static RuleMeta,
    pub level: mago_reporting::Level,
}
