use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Ord, Copy, Clone, Hash, PartialOrd, Deserialize, Serialize)]
#[repr(u8)]
pub enum Category {
    Clarity,
    BestPractices,
    Consistency,
    Deprecation,
    Maintainability,
    Redundancy,
    Security,
    Safety,
    Correctness,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Clarity => "Clarity",
            Category::BestPractices => "Best Practices",
            Category::Consistency => "Consistency",
            Category::Deprecation => "Deprecation",
            Category::Maintainability => "Maintainability",
            Category::Redundancy => "Redundancy",
            Category::Security => "Security",
            Category::Safety => "Safety",
            Category::Correctness => "Correctness",
        }
    }
}
