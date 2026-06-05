use bumpalo::Bump;

use mago_database::file::File;
use mago_syntax::cst::Program;

use crate::ir::IR;
use crate::lower::resolution::namespace::NamespaceResolution;
use crate::lower::resolution::phpdoc::PHPDocResolution;
use crate::lower::resolution::r#type::TypeResolution;

mod argument;
mod assertion_inference;
mod attribute;
mod attribute_target;
mod effect;
mod expression;
mod generics;
mod hook;
mod identifier;
mod inheritance;
mod literal;
mod member;
mod modifier;
mod name;
mod parameter;
mod resolution;
mod settings;
mod statement;
mod r#type;
mod variable;
mod version;

pub use settings::DefineConstantLowering;
pub use settings::LowerSettings;

#[derive(Debug)]
pub struct Lowering<'file, 'arena> {
    pub(crate) arena: &'arena Bump,
    pub(crate) file: &'file File,
    pub(crate) program: &'arena Program<'arena>,
    pub(crate) settings: LowerSettings,
    pub(crate) namespace_resolution: NamespaceResolution<'arena>,
    pub(crate) phpdoc_resolution: PHPDocResolution<'arena>,
    pub(crate) type_resolution: TypeResolution<'arena>,
}

impl<'file, 'arena> Lowering<'file, 'arena> {
    pub fn new(
        arena: &'arena Bump,
        file: &'file File,
        program: &'arena Program<'arena>,
        settings: LowerSettings,
    ) -> Lowering<'file, 'arena> {
        Lowering {
            arena,
            file,
            program,
            settings,
            namespace_resolution: NamespaceResolution::new_in(arena),
            phpdoc_resolution: PHPDocResolution::new(arena, program.trivia.as_slice()),
            type_resolution: TypeResolution::new_in(arena),
        }
    }

    #[must_use]
    pub fn lower(mut self) -> IR<'arena, (), (), ()> {
        let arena = self.arena;
        let program = self.program;

        IR {
            statements: arena
                .alloc_slice_fill_iter(program.statements.iter().map(|statement| self.lower_statement(statement))),
        }
    }
}
