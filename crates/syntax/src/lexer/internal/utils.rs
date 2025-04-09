#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NumberKind {
    Integer,
    Float,
    OctalOrFloat,
    IntegerOrFloat,
}
