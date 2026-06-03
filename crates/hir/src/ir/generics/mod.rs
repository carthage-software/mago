use serde::Serialize;

pub mod annotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum Variance {
    Invariant,
    Covariant,
    Contravariant,
}
