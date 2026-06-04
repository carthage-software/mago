use serde::Serialize;

use crate::ir::statement::Statement;

pub mod argument;
pub mod attribute;
pub mod effect;
pub mod expression;
pub mod flags;
pub mod generics;
pub mod hook;
pub mod identifier;
pub mod inheritance;
pub mod literal;
pub mod member;
pub mod modifier;
pub mod name;
pub mod node;
pub mod parameter;
pub mod statement;
pub mod r#type;
pub mod variable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IR<'arena, S, D, E> {
    pub statements: &'arena [Statement<'arena, S, D, E>],
}
