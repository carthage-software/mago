use bumpalo::Bump;

use mago_syntax::cst::Program;

use crate::ir::IR;
use crate::lower::resolution::namespace::NamespaceResolution;

pub mod argument;
pub mod attribute;
pub mod effect;
pub mod expression;
pub mod generics;
pub mod hook;
pub mod identifier;
pub mod inheritance;
pub mod literal;
pub mod member;
pub mod modifier;
pub mod name;
pub mod parameter;
pub mod resolution;
pub mod statement;
pub mod r#type;
pub mod variable;

#[derive(Debug)]
pub struct Lowering<'arena> {
    pub(crate) arena: &'arena Bump,
    pub(crate) namespace_resolution: NamespaceResolution<'arena>,
}

impl<'arena> Lowering<'arena> {
    pub fn new(arena: &'arena Bump) -> Lowering<'arena> {
        Lowering { arena, namespace_resolution: NamespaceResolution::new_in(arena) }
    }

    #[must_use]
    pub fn lower(mut self, program: &'arena Program<'arena>) -> IR<'arena, (), (), ()> {
        IR {
            statements: self
                .arena
                .alloc_slice_fill_iter(program.statements.iter().map(|statement| self.lower_statement(statement))),
        }
    }
}
