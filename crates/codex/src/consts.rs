/// The maximum number of cases an enum can have before certain expensive
/// analysis.
///
/// This constant acts as a safeguard against combinatorial explosion when analyzing
/// enums with an exceptionally large number of cases.
pub const MAX_ENUM_CASES_FOR_ANALYSIS: usize = 200;
