use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;
use mago_hir::ir::item::expression::anonymous_class::AnonymousClass;
use mago_hir::ir::item::expression::arrow_function::ArrowFunction;
use mago_hir::ir::item::expression::closure::Closure;
use mago_hir::ir::item::statement::class::Class;
use mago_hir::ir::item::statement::constant::Constant;
use mago_hir::ir::item::statement::r#enum::Enum;
use mago_hir::ir::item::statement::function::Function;
use mago_hir::ir::item::statement::interface::Interface;
use mago_hir::ir::item::statement::r#trait::Trait;

pub mod scanner;

/// A definition table is a collection of all the definitions in a program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefinitionTable<'arena, I, S, E> {
    pub constants: &'arena [Constant<'arena, I, S, E>],
    pub functions: &'arena [Function<'arena, I, S, E>],
    pub classes: &'arena [Class<'arena, I, S, E>],
    pub interfaces: &'arena [Interface<'arena, I, S, E>],
    pub traits: &'arena [Trait<'arena, I, S, E>],
    pub enums: &'arena [Enum<'arena, I, S, E>],
    pub anonymous_classes: &'arena [AnonymousClass<'arena, I, S, E>],
    pub closures: &'arena [Closure<'arena, I, S, E>],
    pub arrow_functions: &'arena [ArrowFunction<'arena, I, S, E>],
}

impl<I, S, E> CopyInto for DefinitionTable<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = DefinitionTable<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        DefinitionTable {
            constants: copy_slice_into(self.constants, arena),
            functions: copy_slice_into(self.functions, arena),
            classes: copy_slice_into(self.classes, arena),
            interfaces: copy_slice_into(self.interfaces, arena),
            traits: copy_slice_into(self.traits, arena),
            enums: copy_slice_into(self.enums, arena),
            anonymous_classes: copy_slice_into(self.anonymous_classes, arena),
            closures: copy_slice_into(self.closures, arena),
            arrow_functions: copy_slice_into(self.arrow_functions, arena),
        }
    }
}
