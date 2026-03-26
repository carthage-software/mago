use serde::Serialize;

use crate::category::Category;
use crate::requirements::RuleRequirements;

#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, PartialOrd, Serialize)]
pub struct RuleMeta {
    pub name: &'static str,
    pub code: &'static str,
    pub description: &'static str,
    pub good_example: &'static str,
    pub bad_example: &'static str,
    pub category: Category,
    pub requirements: RuleRequirements,
}

#[derive(Debug, Serialize)]
pub struct RuleEntry {
    #[serde(flatten)]
    pub meta: &'static RuleMeta,
    pub level: mago_reporting::Level,
}
