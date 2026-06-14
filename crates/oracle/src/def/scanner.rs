use mago_allocator::prelude::*;
use mago_hir::ir::IR;
use mago_hir::ir::item::expression::anonymous_class::AnonymousClass;
use mago_hir::ir::item::expression::arrow_function::ArrowFunction;
use mago_hir::ir::item::expression::closure::Closure;
use mago_hir::ir::item::statement::class::Class;
use mago_hir::ir::item::statement::constant::Constant;
use mago_hir::ir::item::statement::r#enum::Enum;
use mago_hir::ir::item::statement::function::Function;
use mago_hir::ir::item::statement::interface::Interface;
use mago_hir::ir::item::statement::r#trait::Trait;
use mago_hir::walker::MutWalker;

use crate::def::DefinitionTable;

/// Scans the IR and collects all definitions into a definition table.
///
/// Generally, I, S, and E are `()` here, but they can be any type that implements `Copy`.
///
/// This is because the IR is generic over these types, and we want to be able to scan any IR regardless of what types it uses for item,
/// statement, or expression metadata. The `Copy` bound ensures that we can safely copy these types when storing them in the definition table.
///
/// ### Parameters
///
/// - `arena`: The arena to allocate the definition table in, must be the same arena as the IR.
/// - `ir`: The IR to scan for definitions.
///
/// ### Returns
///
/// A `DefinitionTable` containing all the definitions found in the IR.
pub fn scan_definitions<'arena, A, I, S, E>(
    arena: &'arena A,
    ir: &'arena IR<'arena, I, S, E>,
) -> DefinitionTable<'arena, I, S, E>
where
    A: Arena,
    I: Copy,
    S: Copy,
    E: Copy,
{
    let mut scanner = Scanner::new_in(arena);
    scanner.walk_ir(ir, &mut ());
    DefinitionTable {
        constants: scanner.constants.leak(),
        functions: scanner.functions.leak(),
        classes: scanner.classes.leak(),
        interfaces: scanner.interfaces.leak(),
        traits: scanner.traits.leak(),
        enums: scanner.enums.leak(),
        anonymous_classes: scanner.anonymous_classes.leak(),
        closures: scanner.closures.leak(),
        arrow_functions: scanner.arrow_functions.leak(),
    }
}

#[derive(Debug)]
struct Scanner<'arena, A: Arena, I, S, E> {
    constants: Vec<'arena, Constant<'arena, I, S, E>, A>,
    functions: Vec<'arena, Function<'arena, I, S, E>, A>,
    classes: Vec<'arena, Class<'arena, I, S, E>, A>,
    interfaces: Vec<'arena, Interface<'arena, I, S, E>, A>,
    traits: Vec<'arena, Trait<'arena, I, S, E>, A>,
    enums: Vec<'arena, Enum<'arena, I, S, E>, A>,
    anonymous_classes: Vec<'arena, AnonymousClass<'arena, I, S, E>, A>,
    closures: Vec<'arena, Closure<'arena, I, S, E>, A>,
    arrow_functions: Vec<'arena, ArrowFunction<'arena, I, S, E>, A>,
}

impl<'arena, A, I, S, E> Scanner<'arena, A, I, S, E>
where
    A: Arena,
{
    pub fn new_in(arena: &'arena A) -> Self {
        Self {
            constants: Vec::new_in(arena),
            functions: Vec::new_in(arena),
            classes: Vec::new_in(arena),
            interfaces: Vec::new_in(arena),
            traits: Vec::new_in(arena),
            enums: Vec::new_in(arena),
            anonymous_classes: Vec::new_in(arena),
            closures: Vec::new_in(arena),
            arrow_functions: Vec::new_in(arena),
        }
    }
}

impl<'arena, A, I, S, E> MutWalker<'arena, I, S, E, ()> for Scanner<'arena, A, I, S, E>
where
    A: Arena,
    I: Copy,
    S: Copy,
    E: Copy,
{
    fn walk_in_constant(&mut self, constant: &'arena Constant<'arena, I, S, E>, _context: &mut ()) {
        self.constants.push(*constant);
    }

    fn walk_in_function(&mut self, function: &'arena Function<'arena, I, S, E>, _context: &mut ()) {
        self.functions.push(*function);
    }

    fn walk_in_class(&mut self, class: &'arena Class<'arena, I, S, E>, _context: &mut ()) {
        self.classes.push(*class);
    }

    fn walk_in_interface(&mut self, interface: &'arena Interface<'arena, I, S, E>, _context: &mut ()) {
        self.interfaces.push(*interface);
    }

    fn walk_in_trait(&mut self, r#trait: &'arena Trait<'arena, I, S, E>, _context: &mut ()) {
        self.traits.push(*r#trait);
    }

    fn walk_in_enum(&mut self, r#enum: &'arena Enum<'arena, I, S, E>, _context: &mut ()) {
        self.enums.push(*r#enum);
    }

    fn walk_in_anonymous_class(&mut self, anonymous_class: &'arena AnonymousClass<'arena, I, S, E>, _context: &mut ()) {
        self.anonymous_classes.push(*anonymous_class);
    }

    fn walk_in_closure(&mut self, closure: &'arena Closure<'arena, I, S, E>, _context: &mut ()) {
        self.closures.push(*closure);
    }

    fn walk_in_arrow_function(&mut self, arrow_function: &'arena ArrowFunction<'arena, I, S, E>, _context: &mut ()) {
        self.arrow_functions.push(*arrow_function);
    }
}
