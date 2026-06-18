use itoa::Buffer as IntegerBuffer;

use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::cst::Program;

use crate::ir::IR;
use crate::ir::variable::DirectVariable;
use crate::lower::error::lower_parse_error;
use crate::lower::interner::Interner;
use crate::lower::resolution::namespace::NamespaceResolution;
use crate::lower::resolution::phpdoc::PHPDocResolution;
use crate::lower::resolution::r#type::TypeResolution;

mod argument;
mod assertion_inference;
mod error;
mod expression;
mod identifier;
mod interner;
mod item;
mod literal;
mod name;
mod resolution;
mod settings;
mod statement;
mod r#type;
mod variable;
mod version;

pub use settings::DefineConstantLowering;
pub use settings::LowerSettings;

#[derive(Debug)]
pub(crate) struct BodyEffects<'arena, A>
where
    A: Arena,
{
    pub(crate) throws: bool,
    pub(crate) yields: bool,
    pub(crate) accessed_globals: Vec<'arena, DirectVariable<'arena>, A>,
}

impl<'arena, A> BodyEffects<'arena, A>
where
    A: Arena,
{
    fn new(arena: &'arena A) -> Self {
        BodyEffects { throws: false, yields: false, accessed_globals: Vec::new_in(arena) }
    }
}

#[derive(Debug)]
pub struct Lowering<'file, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) arena: &'arena A,
    pub(crate) scratch: &'scratch S,
    pub(crate) file: &'file File,
    pub(crate) program: &'scratch Program<'scratch>,
    pub(crate) settings: LowerSettings,
    pub(crate) namespace_resolution: NamespaceResolution<'scratch, S>,
    pub(crate) phpdoc_resolution: PHPDocResolution<'scratch, S>,
    pub(crate) type_resolution: TypeResolution<'scratch, 'arena, S>,
    pub(crate) body_effects: BodyEffects<'arena, A>,
    pub(crate) interner: Interner<'scratch, 'arena, S, A>,
}

impl<'file, 'scratch, 'arena, S, A> Lowering<'file, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub fn new(
        arena: &'arena A,
        scratch: &'scratch S,
        file: &'file File,
        program: &'scratch Program<'scratch>,
        settings: LowerSettings,
    ) -> Lowering<'file, 'scratch, 'arena, S, A> {
        Lowering {
            arena,
            scratch,
            file,
            program,
            settings,
            namespace_resolution: NamespaceResolution::new_in(scratch),
            phpdoc_resolution: PHPDocResolution::new(scratch, program.trivia.as_slice(), settings.process_annotations),
            type_resolution: TypeResolution::new_in(scratch),
            body_effects: BodyEffects::new(arena),
            interner: Interner::new(arena, scratch),
        }
    }

    pub(crate) fn enter_function_like_body(&mut self) -> BodyEffects<'arena, A> {
        std::mem::replace(&mut self.body_effects, BodyEffects::new(self.arena))
    }

    pub(crate) fn leave_function_like_body(&mut self, outer: BodyEffects<'arena, A>) -> BodyEffects<'arena, A> {
        std::mem::replace(&mut self.body_effects, outer)
    }

    /// Builds the synthetic display name for an anonymous construct (closure,
    /// arrow function, or anonymous class).
    ///
    /// Format: `{<prefix>:<workspace-relative path>:<line>:<column>}`, with
    /// 1-based line and column computed from the span's start offset, matching
    /// PHP's own `{closure:...}` stringification.
    pub(crate) fn build_synthetic_name(&self, prefix: &[u8], span: Span) -> &'arena [u8] {
        let line = self.file.line_number(span.start.offset).saturating_add(1);
        let column = self.file.column_number(span.start.offset).saturating_add(1);

        let mut line_buffer = IntegerBuffer::new();
        let mut column_buffer = IntegerBuffer::new();

        let mut name = Vec::new_in(self.arena);
        name.push(b'{');
        name.extend_from_slice(prefix);
        name.push(b':');
        name.extend_from_slice(self.file.name.as_ref());
        name.push(b':');
        name.extend_from_slice(line_buffer.format(line).as_bytes());
        name.push(b':');
        name.extend_from_slice(column_buffer.format(column).as_bytes());
        name.push(b'}');
        name.leak()
    }

    #[must_use]
    pub fn lower(mut self) -> IR<'arena, (), (), ()> {
        let arena = self.arena;
        let program = self.program;

        IR {
            span: program.span(),
            statements: arena
                .alloc_slice_fill_iter(program.statements.iter().map(|statement| self.lower_statement(statement))),
            errors: arena.alloc_slice_fill_iter(program.errors.iter().map(lower_parse_error)),
        }
    }
}
