use mago_allocator::Arena;
use mago_hir::fold::MutItemFold;
use mago_hir::ir::IR;
use mago_hir::ir::item::expression::ItemExpression;
use mago_hir::ir::item::expression::ItemExpressionKind;
use mago_hir::ir::item::member::MemberItemKind;
use mago_hir::ir::item::statement::ItemStatement;
use mago_hir::ir::item::statement::ItemStatementKind;
use mago_span::Span;

use crate::definition::DefinitionTable;
use crate::id::SymbolId;
use crate::symbol::part::origin::Origin;

/// Binds an untyped IR: assigns a [`SymbolId`] to every definition's item-meta
/// slot and indexing each top-level/expression-level definition by that id.
///
/// The result is the same tree with its item-meta hole filled (`IR<'arena,
/// SymbolId, S, E>`) plus a [`DefinitionTable`] keyed by [`SymbolId`]. Members
/// (methods, properties, class constants, enum cases) receive their id in the
/// tree but are not indexed in the table; they live inside their class.
pub fn bind<'arena, A, S, E>(
    arena: &'arena A,
    origin: Origin,
    ir: &'arena IR<'arena, (), S, E>,
) -> (IR<'arena, SymbolId, S, E>, DefinitionTable<'arena, A, S, E>)
where
    A: Arena,
    S: Copy,
    E: Copy,
{
    let mut binder = Binder { arena, current_class: None, table: DefinitionTable::new_in(arena, origin) };
    let ir = binder.fold_ir(ir);

    (ir, binder.table)
}

struct Binder<'arena, A, S, E>
where
    A: Arena,
{
    arena: &'arena A,
    current_class: Option<&'arena [u8]>,
    table: DefinitionTable<'arena, A, S, E>,
}

impl<'arena, A, S, E> MutItemFold<'arena, 'arena, A, S, E> for Binder<'arena, A, S, E>
where
    A: Arena,
    S: Copy,
    E: Copy,
{
    type FromItem = ();
    type ToItem = SymbolId;

    fn arena(&self) -> &'arena A {
        self.arena
    }

    fn fold_item_statement(&mut self, item: &ItemStatement<'arena, (), S, E>) -> ItemStatement<'arena, SymbolId, S, E> {
        let previous = self.current_class;
        self.current_class = class_like_name(&item.kind);
        let kind = self.fold_item_statement_kind(&item.kind);
        self.current_class = previous;

        let meta = self.fold_item_statement_meta(item.span, &kind);

        ItemStatement { meta, span: item.span, kind }
    }

    fn fold_item_expression(
        &mut self,
        item: &ItemExpression<'arena, (), S, E>,
    ) -> ItemExpression<'arena, SymbolId, S, E> {
        let previous = self.current_class;
        if let ItemExpressionKind::AnonymousClass(anonymous_class) = &item.kind {
            self.current_class = Some(anonymous_class.name);
        }
        let kind = self.fold_item_expression_kind(&item.kind);
        self.current_class = previous;

        let meta = self.fold_item_expression_meta(item.span, &kind);

        ItemExpression { meta, span: item.span, kind }
    }

    fn fold_item_statement_meta(&mut self, _span: Span, kind: &ItemStatementKind<'arena, SymbolId, S, E>) -> SymbolId {
        match kind {
            ItemStatementKind::Class(class) => {
                let id = SymbolId::class_like(class.name.value);
                self.table.classes.insert(id, **class);
                id
            }
            ItemStatementKind::Interface(interface) => {
                let id = SymbolId::class_like(interface.name.value);
                self.table.interfaces.insert(id, **interface);
                id
            }
            ItemStatementKind::Trait(trait_definition) => {
                let id = SymbolId::class_like(trait_definition.name.value);
                self.table.traits.insert(id, **trait_definition);
                id
            }
            ItemStatementKind::Enum(enum_definition) => {
                let id = SymbolId::class_like(enum_definition.name.value);
                self.table.enums.insert(id, **enum_definition);
                id
            }
            ItemStatementKind::Constant(constant) => {
                let id = SymbolId::constant(constant.name.value);
                self.table.constants.insert(id, **constant);
                id
            }
            ItemStatementKind::Function(function) => {
                let id = SymbolId::function_like(function.name.value);
                self.table.functions.insert(id, **function);
                id
            }
        }
    }

    fn fold_item_expression_meta(
        &mut self,
        _span: Span,
        kind: &ItemExpressionKind<'arena, SymbolId, S, E>,
    ) -> SymbolId {
        match kind {
            ItemExpressionKind::AnonymousClass(anonymous_class) => {
                let id = SymbolId::class_like(anonymous_class.name);
                self.table.anonymous_classes.insert(id, **anonymous_class);
                id
            }
            ItemExpressionKind::ArrowFunction(arrow_function) => {
                let id = SymbolId::function_like(arrow_function.name);
                self.table.arrow_functions.insert(id, **arrow_function);
                id
            }
            ItemExpressionKind::Closure(closure) => {
                let id = SymbolId::function_like(closure.name);
                self.table.closures.insert(id, **closure);
                id
            }
        }
    }

    fn fold_member_item_meta(&mut self, _span: Span, kind: &MemberItemKind<'arena, SymbolId, S, E>) -> SymbolId {
        let class = self.current_class.unwrap_or(b"");

        match kind {
            MemberItemKind::Method(method) => SymbolId::method(class, method.name.value),
            MemberItemKind::Property(property) => SymbolId::property(class, property.variable.name),
            MemberItemKind::HookedProperty(property) => SymbolId::property(class, property.variable.name),
            MemberItemKind::Constant(constant) => SymbolId::class_like_constant(class, constant.name.value),
            MemberItemKind::EnumCase(case) => SymbolId::enum_case(class, case.name.value),
            MemberItemKind::TraitUse(trait_use) => SymbolId::positional(trait_use.span),
        }
    }
}

/// The fully-qualified name of a class-like item statement, if it is one.
fn class_like_name<'arena, S, E>(kind: &ItemStatementKind<'arena, (), S, E>) -> Option<&'arena [u8]> {
    match kind {
        ItemStatementKind::Class(class) => Some(class.name.value),
        ItemStatementKind::Interface(interface) => Some(interface.name.value),
        ItemStatementKind::Trait(trait_definition) => Some(trait_definition.name.value),
        ItemStatementKind::Enum(enum_definition) => Some(enum_definition.name.value),
        ItemStatementKind::Constant(_) | ItemStatementKind::Function(_) => None,
    }
}
