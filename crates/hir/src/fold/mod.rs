use mago_allocator::Arena;
use mago_span::Span;

use crate::ir::IR;
use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::delimited::Delimited;
use crate::ir::expression::Access;
use crate::ir::expression::AccessKind;
use crate::ir::expression::ArrayElement;
use crate::ir::expression::ArrayElementKind;
use crate::ir::expression::ArrayLike;
use crate::ir::expression::Assignment;
use crate::ir::expression::Binary;
use crate::ir::expression::Call;
use crate::ir::expression::Callee;
use crate::ir::expression::CalleeKind;
use crate::ir::expression::CompositeStringPart;
use crate::ir::expression::Conditional;
use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::Instantiation;
use crate::ir::expression::Match;
use crate::ir::expression::MatchArm;
use crate::ir::expression::MatchArmKind;
use crate::ir::expression::PartialApplication;
use crate::ir::expression::UnaryPostfix;
use crate::ir::expression::UnaryPrefix;
use crate::ir::expression::Yield;
use crate::ir::expression::YieldKind;
use crate::ir::expression::annotation::Annotation;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::ConstantSelectorKind;
use crate::ir::expression::selector::MemberSelector;
use crate::ir::expression::selector::MemberSelectorKind;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::annotation::member::MethodAnnotation;
use crate::ir::item::annotation::parameter::ParameterAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::expression::ItemExpression;
use crate::ir::item::expression::ItemExpressionKind;
use crate::ir::item::expression::anonymous_class::AnonymousClass;
use crate::ir::item::expression::arrow_function::ArrowFunction;
use crate::ir::item::expression::closure::Closure;
use crate::ir::item::member::MemberItem;
use crate::ir::item::member::MemberItemKind;
use crate::ir::item::member::constant::ClassLikeConstant;
use crate::ir::item::member::enum_case::EnumCase;
use crate::ir::item::member::hook::Hook;
use crate::ir::item::member::hook::HookBody;
use crate::ir::item::member::hook::HookBodyKind;
use crate::ir::item::member::method::Method;
use crate::ir::item::member::property::HookedProperty;
use crate::ir::item::member::property::Property;
use crate::ir::item::member::trait_use::TraitUse;
use crate::ir::item::parameter::Parameter;
use crate::ir::item::statement::ItemStatement;
use crate::ir::item::statement::ItemStatementKind;
use crate::ir::item::statement::class::Class;
use crate::ir::item::statement::constant::Constant;
use crate::ir::item::statement::r#enum::Enum;
use crate::ir::item::statement::function::Function;
use crate::ir::item::statement::interface::Interface;
use crate::ir::item::statement::r#trait::Trait;
use crate::ir::statement::Block;
use crate::ir::statement::Declare;
use crate::ir::statement::DeclareItem;
use crate::ir::statement::DoWhile;
use crate::ir::statement::For;
use crate::ir::statement::Foreach;
use crate::ir::statement::GlobalItem;
use crate::ir::statement::If;
use crate::ir::statement::Namespace;
use crate::ir::statement::NamespaceBody;
use crate::ir::statement::Statement;
use crate::ir::statement::StatementKind;
use crate::ir::statement::StaticItem;
use crate::ir::statement::Switch;
use crate::ir::statement::SwitchCase;
use crate::ir::statement::SwitchCaseKind;
use crate::ir::statement::Try;
use crate::ir::statement::TryCatchClause;
use crate::ir::statement::While;
use crate::ir::variable::Variable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

fn fold_delimited<'source, 'arena, T, U, A>(
    arena: &'arena A,
    delimited: &Delimited<'source, T>,
    fold: impl FnMut(&'source T) -> U,
) -> Delimited<'arena, U>
where
    A: Arena,
{
    Delimited { span: delimited.span, items: arena.alloc_slice_fill_iter(delimited.items.iter().map(fold)) }
}

macro_rules! gen_fold_method {
    (mut, ($($from:tt)*), ($($to:tt)*), $node:ident $name:ident $folder:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<fold_ $name>](&mut self, $name: &$node<'source, $($from)*>) -> $node<'arena, $($to)*> {
                let $folder = self;
                $body
            }
        }
    };
    (shared, ($($from:tt)*), ($($to:tt)*), $node:ident $name:ident $folder:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<fold_ $name>](&self, $name: &$node<'source, $($from)*>) -> $node<'arena, $($to)*> {
                let $folder = self;
                $body
            }
        }
    };
}

macro_rules! gen_fold_all_mut {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(mut, (Self::FromItem, Self::FromStatement, Self::FromExpression), (Self::ToItem, Self::ToStatement, Self::ToExpression), $node $name $folder $body);
    };
}
macro_rules! gen_fold_all_const {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(shared, (Self::FromItem, Self::FromStatement, Self::FromExpression), (Self::ToItem, Self::ToStatement, Self::ToExpression), $node $name $folder $body);
    };
}
macro_rules! gen_fold_statement_mut {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(mut, (I, Self::FromStatement, E), (I, Self::ToStatement, E), $node $name $folder $body);
    };
}
macro_rules! gen_fold_statement_const {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(shared, (I, Self::FromStatement, E), (I, Self::ToStatement, E), $node $name $folder $body);
    };
}
macro_rules! gen_fold_expression_mut {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(mut, (I, S, Self::FromExpression), (I, S, Self::ToExpression), $node $name $folder $body);
    };
}
macro_rules! gen_fold_expression_const {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(shared, (I, S, Self::FromExpression), (I, S, Self::ToExpression), $node $name $folder $body);
    };
}
macro_rules! gen_fold_item_mut {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(mut, (Self::FromItem, S, E), (Self::ToItem, S, E), $node $name $folder $body);
    };
}
macro_rules! gen_fold_item_const {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(shared, (Self::FromItem, S, E), (Self::ToItem, S, E), $node $name $folder $body);
    };
}

macro_rules! generate_fold {
    (
        using($folder:ident):
        $( $node:ident as $name:ident => $body:block )*
    ) => {
        /// Folds the [`IR`] with mutable access, transforming all three meta holes at once.
        pub trait MutFold<'source, 'arena, A>
        where
            A: Arena + 'arena,
        {
            type FromItem;
            type FromStatement;
            type FromExpression;
            type ToItem;
            type ToStatement;
            type ToExpression;

            fn arena(&self) -> &'arena A;

            fn fold_statement_meta(&mut self, span: Span, kind: &StatementKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToStatement;
            fn fold_expression_meta(&mut self, span: Span, kind: &ExpressionKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToExpression;
            fn fold_item_statement_meta(&mut self, span: Span, kind: &ItemStatementKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToItem;
            fn fold_item_expression_meta(&mut self, span: Span, kind: &ItemExpressionKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToItem;
            fn fold_member_item_meta(&mut self, span: Span, kind: &MemberItemKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToItem;

            #[inline]
            fn fold_statement(&mut self, statement: &Statement<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> Statement<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_statement_kind(&statement.kind);
                let meta = self.fold_statement_meta(statement.span, &kind);
                Statement { meta, span: statement.span, kind, terminator: statement.terminator }
            }

            #[inline]
            fn fold_expression(&mut self, expression: &Expression<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> Expression<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_expression_kind(&expression.kind);
                let meta = self.fold_expression_meta(expression.span, &kind);
                Expression { span: expression.span, meta, kind }
            }

            #[inline]
            fn fold_item_statement(&mut self, item: &ItemStatement<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> ItemStatement<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_item_statement_kind(&item.kind);
                let meta = self.fold_item_statement_meta(item.span, &kind);
                ItemStatement { meta, span: item.span, kind }
            }

            #[inline]
            fn fold_item_expression(&mut self, item: &ItemExpression<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> ItemExpression<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_item_expression_kind(&item.kind);
                let meta = self.fold_item_expression_meta(item.span, &kind);
                ItemExpression { meta, span: item.span, kind }
            }

            #[inline]
            fn fold_member_item(&mut self, member: &MemberItem<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> MemberItem<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_member_item_kind(&member.kind);
                let meta = self.fold_member_item_meta(member.span, &kind);
                MemberItem { meta, span: member.span, kind, terminator: member.terminator }
            }

            $( gen_fold_all_mut!($node $name $folder $body); )*
        }

        /// Folds the [`IR`] with shared access, transforming all three meta holes at once.
        pub trait Fold<'source, 'arena, A>
        where
            A: Arena + 'arena,
        {
            type FromItem;
            type FromStatement;
            type FromExpression;
            type ToItem;
            type ToStatement;
            type ToExpression;

            fn arena(&self) -> &'arena A;

            fn fold_statement_meta(&self, span: Span, kind: &StatementKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToStatement;
            fn fold_expression_meta(&self, span: Span, kind: &ExpressionKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToExpression;
            fn fold_item_statement_meta(&self, span: Span, kind: &ItemStatementKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToItem;
            fn fold_item_expression_meta(&self, span: Span, kind: &ItemExpressionKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToItem;
            fn fold_member_item_meta(&self, span: Span, kind: &MemberItemKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>) -> Self::ToItem;

            #[inline]
            fn fold_statement(&self, statement: &Statement<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> Statement<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_statement_kind(&statement.kind);
                let meta = self.fold_statement_meta(statement.span, &kind);
                Statement { meta, span: statement.span, kind, terminator: statement.terminator }
            }

            #[inline]
            fn fold_expression(&self, expression: &Expression<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> Expression<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_expression_kind(&expression.kind);
                let meta = self.fold_expression_meta(expression.span, &kind);
                Expression { span: expression.span, meta, kind }
            }

            #[inline]
            fn fold_item_statement(&self, item: &ItemStatement<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> ItemStatement<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_item_statement_kind(&item.kind);
                let meta = self.fold_item_statement_meta(item.span, &kind);
                ItemStatement { meta, span: item.span, kind }
            }

            #[inline]
            fn fold_item_expression(&self, item: &ItemExpression<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> ItemExpression<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_item_expression_kind(&item.kind);
                let meta = self.fold_item_expression_meta(item.span, &kind);
                ItemExpression { meta, span: item.span, kind }
            }

            #[inline]
            fn fold_member_item(&self, member: &MemberItem<'source, Self::FromItem, Self::FromStatement, Self::FromExpression>) -> MemberItem<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression> {
                let kind = self.fold_member_item_kind(&member.kind);
                let meta = self.fold_member_item_meta(member.span, &kind);
                MemberItem { meta, span: member.span, kind, terminator: member.terminator }
            }

            $( gen_fold_all_const!($node $name $folder $body); )*
        }

        /// Folds only the statement meta hole, leaving item and expression metas untouched.
        pub trait MutStatementFold<'source, 'arena, A, I, E>
        where
            A: Arena + 'arena,
            I: Copy,
            E: Copy,
        {
            type FromStatement;
            type ToStatement;

            fn arena(&self) -> &'arena A;

            fn fold_statement_meta(&mut self, span: Span, kind: &StatementKind<'arena, I, Self::ToStatement, E>) -> Self::ToStatement;

            #[inline]
            fn fold_statement(&mut self, statement: &Statement<'source, I, Self::FromStatement, E>) -> Statement<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_statement_kind(&statement.kind);
                let meta = self.fold_statement_meta(statement.span, &kind);
                Statement { meta, span: statement.span, kind, terminator: statement.terminator }
            }

            #[inline]
            fn fold_expression(&mut self, expression: &Expression<'source, I, Self::FromStatement, E>) -> Expression<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_expression_kind(&expression.kind);
                Expression { span: expression.span, meta: expression.meta, kind }
            }

            #[inline]
            fn fold_item_statement(&mut self, item: &ItemStatement<'source, I, Self::FromStatement, E>) -> ItemStatement<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_item_statement_kind(&item.kind);
                ItemStatement { meta: item.meta, span: item.span, kind }
            }

            #[inline]
            fn fold_item_expression(&mut self, item: &ItemExpression<'source, I, Self::FromStatement, E>) -> ItemExpression<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_item_expression_kind(&item.kind);
                ItemExpression { meta: item.meta, span: item.span, kind }
            }

            #[inline]
            fn fold_member_item(&mut self, member: &MemberItem<'source, I, Self::FromStatement, E>) -> MemberItem<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_member_item_kind(&member.kind);
                MemberItem { meta: member.meta, span: member.span, kind, terminator: member.terminator }
            }

            $( gen_fold_statement_mut!($node $name $folder $body); )*
        }

        /// Folds only the statement meta hole, leaving item and expression metas untouched.
        pub trait StatementFold<'source, 'arena, A, I, E>
        where
            A: Arena + 'arena,
            I: Copy,
            E: Copy,
        {
            type FromStatement;
            type ToStatement;

            fn arena(&self) -> &'arena A;

            fn fold_statement_meta(&self, span: Span, kind: &StatementKind<'arena, I, Self::ToStatement, E>) -> Self::ToStatement;

            #[inline]
            fn fold_statement(&self, statement: &Statement<'source, I, Self::FromStatement, E>) -> Statement<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_statement_kind(&statement.kind);
                let meta = self.fold_statement_meta(statement.span, &kind);
                Statement { meta, span: statement.span, kind, terminator: statement.terminator }
            }

            #[inline]
            fn fold_expression(&self, expression: &Expression<'source, I, Self::FromStatement, E>) -> Expression<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_expression_kind(&expression.kind);
                Expression { span: expression.span, meta: expression.meta, kind }
            }

            #[inline]
            fn fold_item_statement(&self, item: &ItemStatement<'source, I, Self::FromStatement, E>) -> ItemStatement<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_item_statement_kind(&item.kind);
                ItemStatement { meta: item.meta, span: item.span, kind }
            }

            #[inline]
            fn fold_item_expression(&self, item: &ItemExpression<'source, I, Self::FromStatement, E>) -> ItemExpression<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_item_expression_kind(&item.kind);
                ItemExpression { meta: item.meta, span: item.span, kind }
            }

            #[inline]
            fn fold_member_item(&self, member: &MemberItem<'source, I, Self::FromStatement, E>) -> MemberItem<'arena, I, Self::ToStatement, E> {
                let kind = self.fold_member_item_kind(&member.kind);
                MemberItem { meta: member.meta, span: member.span, kind, terminator: member.terminator }
            }

            $( gen_fold_statement_const!($node $name $folder $body); )*
        }

        /// Folds only the expression meta hole, leaving item and statement metas untouched.
        pub trait MutExpressionFold<'source, 'arena, A, I, S>
        where
            A: Arena + 'arena,
            I: Copy,
            S: Copy,
        {
            type FromExpression;
            type ToExpression;

            fn arena(&self) -> &'arena A;

            fn fold_expression_meta(&mut self, span: Span, kind: &ExpressionKind<'arena, I, S, Self::ToExpression>) -> Self::ToExpression;

            #[inline]
            fn fold_statement(&mut self, statement: &Statement<'source, I, S, Self::FromExpression>) -> Statement<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_statement_kind(&statement.kind);
                Statement { meta: statement.meta, span: statement.span, kind, terminator: statement.terminator }
            }

            #[inline]
            fn fold_expression(&mut self, expression: &Expression<'source, I, S, Self::FromExpression>) -> Expression<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_expression_kind(&expression.kind);
                let meta = self.fold_expression_meta(expression.span, &kind);
                Expression { span: expression.span, meta, kind }
            }

            #[inline]
            fn fold_item_statement(&mut self, item: &ItemStatement<'source, I, S, Self::FromExpression>) -> ItemStatement<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_item_statement_kind(&item.kind);
                ItemStatement { meta: item.meta, span: item.span, kind }
            }

            #[inline]
            fn fold_item_expression(&mut self, item: &ItemExpression<'source, I, S, Self::FromExpression>) -> ItemExpression<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_item_expression_kind(&item.kind);
                ItemExpression { meta: item.meta, span: item.span, kind }
            }

            #[inline]
            fn fold_member_item(&mut self, member: &MemberItem<'source, I, S, Self::FromExpression>) -> MemberItem<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_member_item_kind(&member.kind);
                MemberItem { meta: member.meta, span: member.span, kind, terminator: member.terminator }
            }

            $( gen_fold_expression_mut!($node $name $folder $body); )*
        }

        /// Folds only the expression meta hole, leaving item and statement metas untouched.
        pub trait ExpressionFold<'source, 'arena, A, I, S>
        where
            A: Arena + 'arena,
            I: Copy,
            S: Copy,
        {
            type FromExpression;
            type ToExpression;

            fn arena(&self) -> &'arena A;

            fn fold_expression_meta(&self, span: Span, kind: &ExpressionKind<'arena, I, S, Self::ToExpression>) -> Self::ToExpression;

            #[inline]
            fn fold_statement(&self, statement: &Statement<'source, I, S, Self::FromExpression>) -> Statement<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_statement_kind(&statement.kind);
                Statement { meta: statement.meta, span: statement.span, kind, terminator: statement.terminator }
            }

            #[inline]
            fn fold_expression(&self, expression: &Expression<'source, I, S, Self::FromExpression>) -> Expression<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_expression_kind(&expression.kind);
                let meta = self.fold_expression_meta(expression.span, &kind);
                Expression { span: expression.span, meta, kind }
            }

            #[inline]
            fn fold_item_statement(&self, item: &ItemStatement<'source, I, S, Self::FromExpression>) -> ItemStatement<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_item_statement_kind(&item.kind);
                ItemStatement { meta: item.meta, span: item.span, kind }
            }

            #[inline]
            fn fold_item_expression(&self, item: &ItemExpression<'source, I, S, Self::FromExpression>) -> ItemExpression<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_item_expression_kind(&item.kind);
                ItemExpression { meta: item.meta, span: item.span, kind }
            }

            #[inline]
            fn fold_member_item(&self, member: &MemberItem<'source, I, S, Self::FromExpression>) -> MemberItem<'arena, I, S, Self::ToExpression> {
                let kind = self.fold_member_item_kind(&member.kind);
                MemberItem { meta: member.meta, span: member.span, kind, terminator: member.terminator }
            }

            $( gen_fold_expression_const!($node $name $folder $body); )*
        }

        /// Folds only the item meta hole, leaving statement and expression metas untouched.
        pub trait MutItemFold<'source, 'arena, A, S, E>
        where
            A: Arena + 'arena,
            S: Copy,
            E: Copy,
        {
            type FromItem;
            type ToItem;

            fn arena(&self) -> &'arena A;

            fn fold_item_statement_meta(&mut self, span: Span, kind: &ItemStatementKind<'arena, Self::ToItem, S, E>) -> Self::ToItem;
            fn fold_item_expression_meta(&mut self, span: Span, kind: &ItemExpressionKind<'arena, Self::ToItem, S, E>) -> Self::ToItem;
            fn fold_member_item_meta(&mut self, span: Span, kind: &MemberItemKind<'arena, Self::ToItem, S, E>) -> Self::ToItem;

            #[inline]
            fn fold_statement(&mut self, statement: &Statement<'source, Self::FromItem, S, E>) -> Statement<'arena, Self::ToItem, S, E> {
                let kind = self.fold_statement_kind(&statement.kind);
                Statement { meta: statement.meta, span: statement.span, kind, terminator: statement.terminator }
            }

            #[inline]
            fn fold_expression(&mut self, expression: &Expression<'source, Self::FromItem, S, E>) -> Expression<'arena, Self::ToItem, S, E> {
                let kind = self.fold_expression_kind(&expression.kind);
                Expression { span: expression.span, meta: expression.meta, kind }
            }

            #[inline]
            fn fold_item_statement(&mut self, item: &ItemStatement<'source, Self::FromItem, S, E>) -> ItemStatement<'arena, Self::ToItem, S, E> {
                let kind = self.fold_item_statement_kind(&item.kind);
                let meta = self.fold_item_statement_meta(item.span, &kind);
                ItemStatement { meta, span: item.span, kind }
            }

            #[inline]
            fn fold_item_expression(&mut self, item: &ItemExpression<'source, Self::FromItem, S, E>) -> ItemExpression<'arena, Self::ToItem, S, E> {
                let kind = self.fold_item_expression_kind(&item.kind);
                let meta = self.fold_item_expression_meta(item.span, &kind);
                ItemExpression { meta, span: item.span, kind }
            }

            #[inline]
            fn fold_member_item(&mut self, member: &MemberItem<'source, Self::FromItem, S, E>) -> MemberItem<'arena, Self::ToItem, S, E> {
                let kind = self.fold_member_item_kind(&member.kind);
                let meta = self.fold_member_item_meta(member.span, &kind);
                MemberItem { meta, span: member.span, kind, terminator: member.terminator }
            }

            $( gen_fold_item_mut!($node $name $folder $body); )*
        }

        /// Folds only the item meta hole, leaving statement and expression metas untouched.
        pub trait ItemFold<'source, 'arena, A, S, E>
        where
            A: Arena + 'arena,
            S: Copy,
            E: Copy,
        {
            type FromItem;
            type ToItem;

            fn arena(&self) -> &'arena A;

            fn fold_item_statement_meta(&self, span: Span, kind: &ItemStatementKind<'arena, Self::ToItem, S, E>) -> Self::ToItem;
            fn fold_item_expression_meta(&self, span: Span, kind: &ItemExpressionKind<'arena, Self::ToItem, S, E>) -> Self::ToItem;
            fn fold_member_item_meta(&self, span: Span, kind: &MemberItemKind<'arena, Self::ToItem, S, E>) -> Self::ToItem;

            #[inline]
            fn fold_statement(&self, statement: &Statement<'source, Self::FromItem, S, E>) -> Statement<'arena, Self::ToItem, S, E> {
                let kind = self.fold_statement_kind(&statement.kind);
                Statement { meta: statement.meta, span: statement.span, kind, terminator: statement.terminator }
            }

            #[inline]
            fn fold_expression(&self, expression: &Expression<'source, Self::FromItem, S, E>) -> Expression<'arena, Self::ToItem, S, E> {
                let kind = self.fold_expression_kind(&expression.kind);
                Expression { span: expression.span, meta: expression.meta, kind }
            }

            #[inline]
            fn fold_item_statement(&self, item: &ItemStatement<'source, Self::FromItem, S, E>) -> ItemStatement<'arena, Self::ToItem, S, E> {
                let kind = self.fold_item_statement_kind(&item.kind);
                let meta = self.fold_item_statement_meta(item.span, &kind);
                ItemStatement { meta, span: item.span, kind }
            }

            #[inline]
            fn fold_item_expression(&self, item: &ItemExpression<'source, Self::FromItem, S, E>) -> ItemExpression<'arena, Self::ToItem, S, E> {
                let kind = self.fold_item_expression_kind(&item.kind);
                let meta = self.fold_item_expression_meta(item.span, &kind);
                ItemExpression { meta, span: item.span, kind }
            }

            #[inline]
            fn fold_member_item(&self, member: &MemberItem<'source, Self::FromItem, S, E>) -> MemberItem<'arena, Self::ToItem, S, E> {
                let kind = self.fold_member_item_kind(&member.kind);
                let meta = self.fold_member_item_meta(member.span, &kind);
                MemberItem { meta, span: member.span, kind, terminator: member.terminator }
            }

            $( gen_fold_item_const!($node $name $folder $body); )*
        }
    };
}

generate_fold! {
    using(folder):

    IR as ir => {
        IR {
            span: ir.span,
            comments: copy_slice_into(ir.comments, folder.arena()),
            statements: folder
                .arena()
                .alloc_slice_fill_iter(ir.statements.iter().map(|statement| folder.fold_statement(statement))),
            errors: folder.arena().alloc_slice_copy(ir.errors),
        }
    }

    StatementKind as statement_kind => {
        match statement_kind {
            StatementKind::Shebang(value) => StatementKind::Shebang(folder.arena().alloc_slice_copy(value)),
            StatementKind::Inline(value) => StatementKind::Inline(folder.arena().alloc_slice_copy(value)),
            StatementKind::Tag(value) => StatementKind::Tag(*value),
            StatementKind::Namespace(node) => {
                StatementKind::Namespace(folder.arena().alloc(folder.fold_namespace(node)))
            }
            StatementKind::Sequence(statements) => StatementKind::Sequence(
                folder.arena().alloc_slice_fill_iter(statements.iter().map(|statement| folder.fold_statement(statement))),
            ),
            StatementKind::Block(node) => StatementKind::Block(folder.arena().alloc(folder.fold_block(node))),
            StatementKind::Item(node) => {
                StatementKind::Item(folder.arena().alloc(folder.fold_item_statement(node)))
            }
            StatementKind::Declare(node) => StatementKind::Declare(folder.arena().alloc(folder.fold_declare(node))),
            StatementKind::Goto(name) => StatementKind::Goto(name.copy_into(folder.arena())),
            StatementKind::Label(name) => StatementKind::Label(name.copy_into(folder.arena())),
            StatementKind::Try(node) => StatementKind::Try(folder.arena().alloc(folder.fold_try_statement(node))),
            StatementKind::Foreach(node) => StatementKind::Foreach(folder.arena().alloc(folder.fold_foreach(node))),
            StatementKind::For(node) => StatementKind::For(folder.arena().alloc(folder.fold_for_loop(node))),
            StatementKind::While(node) => StatementKind::While(folder.arena().alloc(folder.fold_while_loop(node))),
            StatementKind::DoWhile(node) => StatementKind::DoWhile(folder.arena().alloc(folder.fold_do_while(node))),
            StatementKind::Continue(value) => StatementKind::Continue(
                value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
            ),
            StatementKind::Break(value) => {
                StatementKind::Break(value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))))
            }
            StatementKind::Switch(node) => StatementKind::Switch(folder.arena().alloc(folder.fold_switch(node))),
            StatementKind::If(node) => StatementKind::If(folder.arena().alloc(folder.fold_if_statement(node))),
            StatementKind::Return(value) => StatementKind::Return(
                value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
            ),
            StatementKind::Expression(node) => {
                StatementKind::Expression(folder.arena().alloc(folder.fold_expression(node)))
            }
            StatementKind::Echo(values) => StatementKind::Echo(
                folder.arena().alloc_slice_fill_iter(values.iter().map(|value| folder.fold_expression(value))),
            ),
            StatementKind::Use(items) => StatementKind::Use(
                folder.arena().alloc_slice_fill_iter(items.iter().map(|item| item.copy_into(folder.arena()))),
            ),
            StatementKind::Global(items) => StatementKind::Global(
                folder.arena().alloc_slice_fill_iter(items.iter().map(|item| folder.fold_global_item(item))),
            ),
            StatementKind::Static(items) => StatementKind::Static(
                folder.arena().alloc_slice_fill_iter(items.iter().map(|item| folder.fold_static_item(item))),
            ),
            StatementKind::Unset(values) => StatementKind::Unset(fold_delimited(
                folder.arena(),
                values,
                |value| folder.fold_expression(value),
            )),
            StatementKind::VariableBindingAnnotation(node) => {
                StatementKind::VariableBindingAnnotation(copy_ref_into(*node, folder.arena()))
            }
            StatementKind::HaltCompiler => StatementKind::HaltCompiler,
            StatementKind::Noop => StatementKind::Noop,
        }
    }

    Namespace as namespace => {
        Namespace {
            span: namespace.span,
            name: namespace.name.map(|name| copy_ref_into(name, folder.arena())),
            body: match &namespace.body {
                NamespaceBody::BraceDelimited(block) => {
                    NamespaceBody::BraceDelimited(folder.arena().alloc(folder.fold_block(block)))
                }
                NamespaceBody::Implicit { terminator, statements } => NamespaceBody::Implicit {
                    terminator: *terminator,
                    statements: folder
                        .arena()
                        .alloc_slice_fill_iter(statements.iter().map(|statement| folder.fold_statement(statement))),
                },
            },
        }
    }

    Declare as declare => {
        Declare {
            span: declare.span,
            items: fold_delimited(folder.arena(), &declare.items, |item| folder.fold_declare_item(item)),
            statement: folder.arena().alloc(folder.fold_statement(declare.statement)),
        }
    }

    DeclareItem as declare_item => {
        DeclareItem {
            span: declare_item.span,
            name: declare_item.name.copy_into(folder.arena()),
            value: declare_item.value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    Block as block => {
        Block {
            span: block.span,
            statements: folder
                .arena()
                .alloc_slice_fill_iter(block.statements.iter().map(|statement| folder.fold_statement(statement))),
        }
    }

    Try as try_statement => {
        Try {
            span: try_statement.span,
            block: folder.arena().alloc(folder.fold_block(try_statement.block)),
            catch_clauses: folder.arena().alloc_slice_fill_iter(
                try_statement.catch_clauses.iter().map(|clause| folder.fold_try_catch_clause(clause)),
            ),
            finally_block: try_statement
                .finally_block
                .map(|clause| &*folder.arena().alloc(folder.fold_block(clause))),
        }
    }

    TryCatchClause as try_catch_clause => {
        TryCatchClause {
            span: try_catch_clause.span,
            r#type: copy_ref_into(try_catch_clause.r#type, folder.arena()),
            variable: try_catch_clause.variable.map(|variable| variable.copy_into(folder.arena())),
            block: folder.arena().alloc(folder.fold_block(try_catch_clause.block)),
        }
    }

    Foreach as foreach => {
        Foreach {
            span: foreach.span,
            expression: folder.arena().alloc(folder.fold_expression(foreach.expression)),
            key: foreach.key.map(|key| &*folder.arena().alloc(folder.fold_expression(key))),
            value: folder.arena().alloc(folder.fold_expression(foreach.value)),
            statement: folder.arena().alloc(folder.fold_statement(foreach.statement)),
        }
    }

    For as for_loop => {
        For {
            span: for_loop.span,
            initializations: folder.arena().alloc_slice_fill_iter(
                for_loop.initializations.iter().map(|expression| folder.fold_expression(expression)),
            ),
            conditions: folder.arena().alloc_slice_fill_iter(
                for_loop.conditions.iter().map(|expression| folder.fold_expression(expression)),
            ),
            increments: folder.arena().alloc_slice_fill_iter(
                for_loop.increments.iter().map(|expression| folder.fold_expression(expression)),
            ),
            statement: folder.arena().alloc(folder.fold_statement(for_loop.statement)),
        }
    }

    While as while_loop => {
        While {
            span: while_loop.span,
            condition: folder.arena().alloc(folder.fold_expression(while_loop.condition)),
            statement: folder.arena().alloc(folder.fold_statement(while_loop.statement)),
        }
    }

    DoWhile as do_while => {
        DoWhile {
            span: do_while.span,
            statement: folder.arena().alloc(folder.fold_statement(do_while.statement)),
            condition: folder.arena().alloc(folder.fold_expression(do_while.condition)),
        }
    }

    Switch as switch => {
        Switch {
            span: switch.span,
            subject: folder.arena().alloc(folder.fold_expression(switch.subject)),
            cases: fold_delimited(folder.arena(), &switch.cases, |case| folder.fold_switch_case(case)),
        }
    }

    SwitchCase as switch_case => {
        SwitchCase {
            span: switch_case.span,
            seperator: switch_case.seperator,
            kind: match &switch_case.kind {
                SwitchCaseKind::Expression(expression, statements) => SwitchCaseKind::Expression(
                    folder.arena().alloc(folder.fold_expression(expression)),
                    folder.arena().alloc_slice_fill_iter(statements.iter().map(|statement| folder.fold_statement(statement))),
                ),
                SwitchCaseKind::Default(statements) => SwitchCaseKind::Default(
                    folder.arena().alloc_slice_fill_iter(statements.iter().map(|statement| folder.fold_statement(statement))),
                ),
            },
        }
    }

    If as if_statement => {
        If {
            span: if_statement.span,
            condition: folder.arena().alloc(folder.fold_expression(if_statement.condition)),
            then: folder.arena().alloc(folder.fold_statement(if_statement.then)),
            r#else: if_statement.r#else.map(|statement| &*folder.arena().alloc(folder.fold_statement(statement))),
        }
    }

    StaticItem as static_item => {
        StaticItem {
            span: static_item.span,
            variable: static_item.variable.copy_into(folder.arena()),
            type_annotation: static_item
                .type_annotation
                .map(|type_annotation| copy_ref_into(type_annotation, folder.arena())),
            value: static_item.value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    GlobalItem as global_item => {
        GlobalItem {
            span: global_item.span,
            variable: folder.fold_variable(&global_item.variable),
            type_annotation: global_item
                .type_annotation
                .map(|type_annotation| copy_ref_into(type_annotation, folder.arena())),
        }
    }

    ItemStatementKind as item_statement_kind => {
        match item_statement_kind {
            ItemStatementKind::Class(node) => {
                ItemStatementKind::Class(folder.arena().alloc(folder.fold_class(node)))
            }
            ItemStatementKind::Interface(node) => {
                ItemStatementKind::Interface(folder.arena().alloc(folder.fold_interface(node)))
            }
            ItemStatementKind::Trait(node) => {
                ItemStatementKind::Trait(folder.arena().alloc(folder.fold_trait_definition(node)))
            }
            ItemStatementKind::Enum(node) => {
                ItemStatementKind::Enum(folder.arena().alloc(folder.fold_enum_definition(node)))
            }
            ItemStatementKind::Constant(node) => {
                ItemStatementKind::Constant(folder.arena().alloc(folder.fold_constant(node)))
            }
            ItemStatementKind::Function(node) => {
                ItemStatementKind::Function(folder.arena().alloc(folder.fold_function(node)))
            }
        }
    }

    Class as class => {
        Class {
            span: class.span,
            annotation: class.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(class.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            version_constraint: folder.arena().alloc_slice_copy(class.version_constraint),
            attribute_target: class.attribute_target,
            modifiers: folder.arena().alloc_slice_copy(class.modifiers),
            name: class.name.copy_into(folder.arena()),
            extends: class.extends.map(|extends| copy_ref_into(extends, folder.arena())),
            implements: class.implements.map(|implements| copy_ref_into(implements, folder.arena())),
            members: fold_delimited(folder.arena(), &class.members, |member| folder.fold_member_item(member)),
        }
    }

    Interface as interface => {
        Interface {
            span: interface.span,
            annotation: interface.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(interface.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            version_constraint: folder.arena().alloc_slice_copy(interface.version_constraint),
            name: interface.name.copy_into(folder.arena()),
            extends: interface.extends.map(|extends| copy_ref_into(extends, folder.arena())),
            members: fold_delimited(folder.arena(), &interface.members, |member| folder.fold_member_item(member)),
        }
    }

    Trait as trait_definition => {
        Trait {
            span: trait_definition.span,
            annotation: trait_definition.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder.arena().alloc_slice_fill_iter(
                trait_definition.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            version_constraint: folder.arena().alloc_slice_copy(trait_definition.version_constraint),
            name: trait_definition.name.copy_into(folder.arena()),
            members: fold_delimited(folder.arena(), &trait_definition.members, |member| folder.fold_member_item(member)),
        }
    }

    Enum as enum_definition => {
        Enum {
            span: enum_definition.span,
            annotation: enum_definition.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder.arena().alloc_slice_fill_iter(
                enum_definition.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            version_constraint: folder.arena().alloc_slice_copy(enum_definition.version_constraint),
            name: enum_definition.name.copy_into(folder.arena()),
            backing_type: enum_definition.backing_type.map(|backing_type| backing_type.copy_into(folder.arena())),
            implements: enum_definition.implements.map(|implements| copy_ref_into(implements, folder.arena())),
            members: fold_delimited(folder.arena(), &enum_definition.members, |member| folder.fold_member_item(member)),
        }
    }

    Constant as constant => {
        Constant {
            span: constant.span,
            annotation: constant.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(constant.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            version_constraint: folder.arena().alloc_slice_copy(constant.version_constraint),
            name: constant.name.copy_into(folder.arena()),
            value: folder.arena().alloc(folder.fold_expression(constant.value)),
            flattened: constant.flattened,
        }
    }

    Function as function => {
        Function {
            span: function.span,
            annotation: function.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(function.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            flags: function.flags,
            version_constraint: folder.arena().alloc_slice_copy(function.version_constraint),
            name: function.name.copy_into(folder.arena()),
            parameters: fold_delimited(folder.arena(), &function.parameters, |parameter| folder.fold_parameter(parameter)),
            return_type: function.return_type.map(|return_type| copy_ref_into(return_type, folder.arena())),
            body: folder.arena().alloc(folder.fold_block(function.body)),
            direct_accessed_globals: copy_slice_into(function.direct_accessed_globals, folder.arena()),
        }
    }

    MemberItemKind as member_item_kind => {
        match member_item_kind {
            MemberItemKind::Method(node) => MemberItemKind::Method(folder.arena().alloc(folder.fold_method(node))),
            MemberItemKind::Property(node) => MemberItemKind::Property(folder.arena().alloc(folder.fold_property(node))),
            MemberItemKind::HookedProperty(node) => {
                MemberItemKind::HookedProperty(folder.arena().alloc(folder.fold_hooked_property(node)))
            }
            MemberItemKind::TraitUse(node) => MemberItemKind::TraitUse(folder.arena().alloc(folder.fold_trait_use(node))),
            MemberItemKind::Constant(node) => {
                MemberItemKind::Constant(folder.arena().alloc(folder.fold_class_like_constant(node)))
            }
            MemberItemKind::EnumCase(node) => MemberItemKind::EnumCase(folder.arena().alloc(folder.fold_enum_case(node))),
        }
    }

    Method as method => {
        Method {
            span: method.span,
            annotation: method.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(method.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            version_constraint: folder.arena().alloc_slice_copy(method.version_constraint),
            flags: method.flags,
            modifiers: folder.arena().alloc_slice_copy(method.modifiers),
            name: method.name.copy_into(folder.arena()),
            parameters: fold_delimited(folder.arena(), &method.parameters, |parameter| folder.fold_parameter(parameter)),
            return_type: method.return_type.map(|return_type| copy_ref_into(return_type, folder.arena())),
            body: method.body.map(|body| &*folder.arena().alloc(folder.fold_block(body))),
            direct_accessed_globals: copy_slice_into(method.direct_accessed_globals, folder.arena()),
        }
    }

    Property as property => {
        Property {
            span: property.span,
            annotation: property.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(property.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            version_constraint: folder.arena().alloc_slice_copy(property.version_constraint),
            modifiers: folder.arena().alloc_slice_copy(property.modifiers),
            r#type: property.r#type.map(|r#type| copy_ref_into(r#type, folder.arena())),
            variable: property.variable.copy_into(folder.arena()),
            default_value: property.default_value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
            flattened: property.flattened,
        }
    }

    HookedProperty as hooked_property => {
        HookedProperty {
            span: hooked_property.span,
            annotation: hooked_property.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder.arena().alloc_slice_fill_iter(
                hooked_property.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            version_constraint: folder.arena().alloc_slice_copy(hooked_property.version_constraint),
            modifiers: folder.arena().alloc_slice_copy(hooked_property.modifiers),
            r#type: hooked_property.r#type.map(|r#type| copy_ref_into(r#type, folder.arena())),
            variable: hooked_property.variable.copy_into(folder.arena()),
            default_value: hooked_property.default_value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
            hooks: fold_delimited(folder.arena(), &hooked_property.hooks, |hook| folder.fold_hook(hook)),
        }
    }

    ClassLikeConstant as class_like_constant => {
        ClassLikeConstant {
            span: class_like_constant.span,
            annotation: class_like_constant.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder.arena().alloc_slice_fill_iter(
                class_like_constant.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            version_constraint: folder.arena().alloc_slice_copy(class_like_constant.version_constraint),
            modifiers: folder.arena().alloc_slice_copy(class_like_constant.modifiers),
            r#type: class_like_constant.r#type.map(|r#type| copy_ref_into(r#type, folder.arena())),
            name: class_like_constant.name.copy_into(folder.arena()),
            value: folder.arena().alloc(folder.fold_expression(class_like_constant.value)),
            flattened: class_like_constant.flattened,
        }
    }

    EnumCase as enum_case => {
        EnumCase {
            span: enum_case.span,
            annotation: enum_case.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(enum_case.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            version_constraint: folder.arena().alloc_slice_copy(enum_case.version_constraint),
            name: enum_case.name.copy_into(folder.arena()),
            value: enum_case.value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    TraitUse as trait_use => {
        TraitUse {
            span: trait_use.span,
            annotation: trait_use.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            traits: copy_slice_into(trait_use.traits, folder.arena()),
            adaptations: trait_use.adaptations.map(|adaptations| adaptations.copy_into(folder.arena())),
        }
    }

    Hook as hook => {
        Hook {
            span: hook.span,
            annotation: hook.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(hook.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            version_constraint: folder.arena().alloc_slice_copy(hook.version_constraint),
            flags: hook.flags,
            modifiers: folder.arena().alloc_slice_copy(hook.modifiers),
            name: hook.name.copy_into(folder.arena()),
            parameters: hook
                .parameters
                .as_ref()
                .map(|parameters| fold_delimited(folder.arena(), parameters, |parameter| folder.fold_parameter(parameter))),
            body: hook.body.as_ref().map(|body| folder.fold_hook_body(body)),
        }
    }

    HookBody as hook_body => {
        HookBody { span: hook_body.span, kind: folder.fold_hook_body_kind(&hook_body.kind) }
    }

    HookBodyKind as hook_body_kind => {
        match hook_body_kind {
            HookBodyKind::Expression(expression) => {
                HookBodyKind::Expression(folder.arena().alloc(folder.fold_expression(expression)))
            }
            HookBodyKind::Block(block) => HookBodyKind::Block(folder.arena().alloc(folder.fold_block(block))),
        }
    }

    Parameter as parameter => {
        Parameter {
            span: parameter.span,
            annotation: parameter.annotation.map(|annotation| copy_ref_into(annotation, folder.arena())),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(parameter.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            flags: parameter.flags,
            version_constraint: folder.arena().alloc_slice_copy(parameter.version_constraint),
            modifiers: folder.arena().alloc_slice_copy(parameter.modifiers),
            r#type: parameter.r#type.map(|r#type| copy_ref_into(r#type, folder.arena())),
            variable: parameter.variable.copy_into(folder.arena()),
            default_value: parameter.default_value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
            hooks: parameter
                .hooks
                .as_ref()
                .map(|hooks| fold_delimited(folder.arena(), hooks, |hook| folder.fold_hook(hook))),
        }
    }

    ItemAnnotation as item_annotation => {
        ItemAnnotation {
            span: item_annotation.span,
            type_aliases: copy_slice_into(item_annotation.type_aliases, folder.arena()),
            imported_type_aliases: copy_slice_into(item_annotation.imported_type_aliases, folder.arena()),
            type_parameters: copy_slice_into(item_annotation.type_parameters, folder.arena()),
            inherited_type_parameters: copy_slice_into(item_annotation.inherited_type_parameters, folder.arena()),
            extends: copy_slice_into(item_annotation.extends, folder.arena()),
            require_extends: copy_slice_into(item_annotation.require_extends, folder.arena()),
            implements: copy_slice_into(item_annotation.implements, folder.arena()),
            require_implements: copy_slice_into(item_annotation.require_implements, folder.arena()),
            uses: copy_slice_into(item_annotation.uses, folder.arena()),
            sealings: copy_slice_into(item_annotation.sealings, folder.arena()),
            mixins: copy_slice_into(item_annotation.mixins, folder.arena()),
            methods: folder.arena().alloc_slice_fill_iter(
                item_annotation.methods.iter().map(|method| folder.fold_method_annotation(method)),
            ),
            properties: copy_slice_into(item_annotation.properties, folder.arena()),
            parameters: folder.arena().alloc_slice_fill_iter(
                item_annotation.parameters.iter().map(|parameter| folder.fold_parameter_annotation(parameter)),
            ),
            parameter_outs: copy_slice_into(item_annotation.parameter_outs, folder.arena()),
            where_constraints: copy_slice_into(item_annotation.where_constraints, folder.arena()),
            return_type: copy_slice_into(item_annotation.return_type, folder.arena()),
            throws: copy_slice_into(item_annotation.throws, folder.arena()),
            asserts: copy_slice_into(item_annotation.asserts, folder.arena()),
            asserts_if_true: copy_slice_into(item_annotation.asserts_if_true, folder.arena()),
            asserts_if_false: copy_slice_into(item_annotation.asserts_if_false, folder.arena()),
            self_out: copy_slice_into(item_annotation.self_out, folder.arena()),
            pure_unless_callable_impure: copy_slice_into(item_annotation.pure_unless_callable_impure, folder.arena()),
            var: copy_slice_into(item_annotation.var, folder.arena()),
            tags: item_annotation.tags,
            errors: folder.arena().alloc_slice_copy(item_annotation.errors),
        }
    }

    MethodAnnotation as method_annotation => {
        MethodAnnotation {
            span: method_annotation.span,
            visibility: method_annotation.visibility,
            r#static: method_annotation.r#static,
            name: method_annotation.name.copy_into(folder.arena()),
            type_parameters: method_annotation
                .type_parameters
                .map(|type_parameters| type_parameters.copy_into(folder.arena())),
            parameters: fold_delimited(folder.arena(), &method_annotation.parameters, |parameter| {
                folder.fold_parameter_annotation(parameter)
            }),
            return_type: method_annotation.return_type.map(|return_type| copy_ref_into(return_type, folder.arena())),
        }
    }

    ParameterAnnotation as parameter_annotation => {
        ParameterAnnotation {
            span: parameter_annotation.span,
            r#type: parameter_annotation.r#type.map(|r#type| copy_ref_into(r#type, folder.arena())),
            is_by_reference: parameter_annotation.is_by_reference,
            is_variadic: parameter_annotation.is_variadic,
            variable: parameter_annotation.variable.map(|variable| variable.copy_into(folder.arena())),
            default_value: parameter_annotation
                .default_value
                .map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    Attribute as attribute => {
        Attribute {
            span: attribute.span,
            class: attribute.class.copy_into(folder.arena()),
            arguments: attribute
                .arguments
                .as_ref()
                .map(|arguments| fold_delimited(folder.arena(), arguments, |argument| folder.fold_partial_argument(argument))),
        }
    }

    Argument as argument => {
        match argument {
            Argument::Value(expression) => Argument::Value(folder.arena().alloc(folder.fold_expression(expression))),
            Argument::Variadic(expression) => {
                Argument::Variadic(folder.arena().alloc(folder.fold_expression(expression)))
            }
            Argument::Named(name, expression) => Argument::Named(
                name.copy_into(folder.arena()),
                folder.arena().alloc(folder.fold_expression(expression)),
            ),
        }
    }

    PartialArgument as partial_argument => {
        match partial_argument {
            PartialArgument::Value(expression) => {
                PartialArgument::Value(folder.arena().alloc(folder.fold_expression(expression)))
            }
            PartialArgument::Variadic(expression) => {
                PartialArgument::Variadic(folder.arena().alloc(folder.fold_expression(expression)))
            }
            PartialArgument::Named(name, expression) => PartialArgument::Named(
                name.copy_into(folder.arena()),
                folder.arena().alloc(folder.fold_expression(expression)),
            ),
            PartialArgument::Placeholder(span) => PartialArgument::Placeholder(*span),
            PartialArgument::NamedPlaceholder(name) => {
                PartialArgument::NamedPlaceholder(name.copy_into(folder.arena()))
            }
            PartialArgument::VariadicPlaceholder(span) => PartialArgument::VariadicPlaceholder(*span),
        }
    }

    ExpressionKind as expression_kind => {
        match expression_kind {
            ExpressionKind::Parenthesized(node) => {
                ExpressionKind::Parenthesized(folder.arena().alloc(folder.fold_expression(node)))
            }
            ExpressionKind::Binary(node) => ExpressionKind::Binary(folder.arena().alloc(folder.fold_binary(node))),
            ExpressionKind::UnaryPrefix(node) => {
                ExpressionKind::UnaryPrefix(folder.arena().alloc(folder.fold_unary_prefix(node)))
            }
            ExpressionKind::UnaryPostfix(node) => {
                ExpressionKind::UnaryPostfix(folder.arena().alloc(folder.fold_unary_postfix(node)))
            }
            ExpressionKind::Literal(node) => ExpressionKind::Literal(copy_ref_into(*node, folder.arena())),
            ExpressionKind::CompositeString(parts) => ExpressionKind::CompositeString(
                folder.arena().alloc_slice_fill_iter(parts.iter().map(|part| folder.fold_composite_string_part(part))),
            ),
            ExpressionKind::ShellExecute(parts) => ExpressionKind::ShellExecute(
                folder.arena().alloc_slice_fill_iter(parts.iter().map(|part| folder.fold_composite_string_part(part))),
            ),
            ExpressionKind::Assignment(node) => {
                ExpressionKind::Assignment(folder.arena().alloc(folder.fold_assignment(node)))
            }
            ExpressionKind::Annotation(node) => {
                ExpressionKind::Annotation(folder.arena().alloc(folder.fold_annotation(node)))
            }
            ExpressionKind::Conditional(node) => {
                ExpressionKind::Conditional(folder.arena().alloc(folder.fold_conditional(node)))
            }
            ExpressionKind::ArrayLike(array_like) => ExpressionKind::ArrayLike(ArrayLike {
                span: array_like.span,
                kind: array_like.kind,
                elements: fold_delimited(folder.arena(), &array_like.elements, |element| {
                    folder.fold_array_element(element)
                }),
            }),
            ExpressionKind::ArrayAppend(node) => {
                ExpressionKind::ArrayAppend(folder.arena().alloc(folder.fold_expression(node)))
            }
            ExpressionKind::Item(node) => {
                ExpressionKind::Item(folder.arena().alloc(folder.fold_item_expression(node)))
            }
            ExpressionKind::Call(node) => ExpressionKind::Call(folder.arena().alloc(folder.fold_call(node))),
            ExpressionKind::PartialApplication(node) => {
                ExpressionKind::PartialApplication(folder.arena().alloc(folder.fold_partial_application(node)))
            }
            ExpressionKind::Access(node) => ExpressionKind::Access(folder.arena().alloc(folder.fold_access(node))),
            ExpressionKind::Clone(node) => ExpressionKind::Clone(folder.arena().alloc(folder.fold_expression(node))),
            ExpressionKind::Empty(node) => ExpressionKind::Empty(folder.arena().alloc(folder.fold_expression(node))),
            ExpressionKind::Eval(node) => ExpressionKind::Eval(folder.arena().alloc(folder.fold_expression(node))),
            ExpressionKind::Include(node) => ExpressionKind::Include(folder.arena().alloc(folder.fold_expression(node))),
            ExpressionKind::IncludeOnce(node) => {
                ExpressionKind::IncludeOnce(folder.arena().alloc(folder.fold_expression(node)))
            }
            ExpressionKind::Require(node) => ExpressionKind::Require(folder.arena().alloc(folder.fold_expression(node))),
            ExpressionKind::RequireOnce(node) => {
                ExpressionKind::RequireOnce(folder.arena().alloc(folder.fold_expression(node)))
            }
            ExpressionKind::Print(node) => ExpressionKind::Print(folder.arena().alloc(folder.fold_expression(node))),
            ExpressionKind::Isset(values) => ExpressionKind::Isset(fold_delimited(
                folder.arena(),
                values,
                |value| folder.fold_expression(value),
            )),
            ExpressionKind::Exit(arguments) => ExpressionKind::Exit(arguments.as_ref().map(|arguments| {
                fold_delimited(folder.arena(), arguments, |argument| folder.fold_argument(argument))
            })),
            ExpressionKind::MagicConstant(value) => ExpressionKind::MagicConstant(*value),
            ExpressionKind::Constant(identifier) => ExpressionKind::Constant(identifier.copy_into(folder.arena())),
            ExpressionKind::Instantiation(node) => {
                ExpressionKind::Instantiation(folder.arena().alloc(folder.fold_instantiation(node)))
            }
            ExpressionKind::Variable(variable) => ExpressionKind::Variable(folder.fold_variable(variable)),
            ExpressionKind::Yield(node) => {
                ExpressionKind::Yield(folder.arena().alloc(folder.fold_yield_expression(node)))
            }
            ExpressionKind::Throw(node) => ExpressionKind::Throw(folder.arena().alloc(folder.fold_expression(node))),
            ExpressionKind::Match(node) => {
                ExpressionKind::Match(folder.arena().alloc(folder.fold_match_expression(node)))
            }
            ExpressionKind::Identifier(identifier) => ExpressionKind::Identifier(identifier.copy_into(folder.arena())),
            ExpressionKind::Parent => ExpressionKind::Parent,
            ExpressionKind::Self_ => ExpressionKind::Self_,
            ExpressionKind::Static => ExpressionKind::Static,
            ExpressionKind::Error(span) => ExpressionKind::Error(*span),
        }
    }

    Assignment as assignment => {
        Assignment {
            span: assignment.span,
            left: folder.arena().alloc(folder.fold_expression(assignment.left)),
            operator: assignment.operator,
            right: folder.arena().alloc(folder.fold_expression(assignment.right)),
        }
    }

    Annotation as annotation => {
        Annotation {
            annotation: copy_ref_into(annotation.annotation, folder.arena()),
            expression: folder.arena().alloc(folder.fold_expression(annotation.expression)),
        }
    }

    Binary as binary => {
        Binary {
            span: binary.span,
            left: folder.arena().alloc(folder.fold_expression(binary.left)),
            operator: binary.operator,
            right: folder.arena().alloc(folder.fold_expression(binary.right)),
        }
    }

    UnaryPrefix as unary_prefix => {
        UnaryPrefix {
            span: unary_prefix.span,
            operator: unary_prefix.operator,
            operand: folder.arena().alloc(folder.fold_expression(unary_prefix.operand)),
        }
    }

    UnaryPostfix as unary_postfix => {
        UnaryPostfix {
            span: unary_postfix.span,
            operand: folder.arena().alloc(folder.fold_expression(unary_postfix.operand)),
            operator: unary_postfix.operator,
        }
    }

    Conditional as conditional => {
        Conditional {
            span: conditional.span,
            condition: folder.arena().alloc(folder.fold_expression(conditional.condition)),
            then: conditional.then.map(|then| &*folder.arena().alloc(folder.fold_expression(then))),
            r#else: folder.arena().alloc(folder.fold_expression(conditional.r#else)),
        }
    }

    ArrayElement as array_element => {
        ArrayElement { span: array_element.span, kind: folder.fold_array_element_kind(&array_element.kind) }
    }

    ArrayElementKind as array_element_kind => {
        match array_element_kind {
            ArrayElementKind::KeyValue(key, value) => ArrayElementKind::KeyValue(
                folder.arena().alloc(folder.fold_expression(key)),
                folder.arena().alloc(folder.fold_expression(value)),
            ),
            ArrayElementKind::Value(value) => ArrayElementKind::Value(folder.arena().alloc(folder.fold_expression(value))),
            ArrayElementKind::Variadic(value) => {
                ArrayElementKind::Variadic(folder.arena().alloc(folder.fold_expression(value)))
            }
            ArrayElementKind::Missing => ArrayElementKind::Missing,
        }
    }

    CompositeStringPart as composite_string_part => {
        match composite_string_part {
            CompositeStringPart::Literal(value) => {
                CompositeStringPart::Literal(folder.arena().alloc_slice_copy(value))
            }
            CompositeStringPart::Expression(expression) => {
                CompositeStringPart::Expression(folder.arena().alloc(folder.fold_expression(expression)))
            }
        }
    }

    Instantiation as instantiation => {
        Instantiation {
            span: instantiation.span,
            class: folder.arena().alloc(folder.fold_expression(instantiation.class)),
            arguments: instantiation
                .arguments
                .as_ref()
                .map(|arguments| fold_delimited(folder.arena(), arguments, |argument| folder.fold_argument(argument))),
        }
    }

    Call as call => {
        Call {
            span: call.span,
            callee: folder.fold_callee(&call.callee),
            arguments: fold_delimited(folder.arena(), &call.arguments, |argument| folder.fold_argument(argument)),
        }
    }

    PartialApplication as partial_application => {
        PartialApplication {
            span: partial_application.span,
            callee: folder.fold_callee(&partial_application.callee),
            arguments: fold_delimited(folder.arena(), &partial_application.arguments, |argument| {
                folder.fold_partial_argument(argument)
            }),
        }
    }

    Callee as callee => {
        Callee { span: callee.span, kind: folder.fold_callee_kind(&callee.kind) }
    }

    CalleeKind as callee_kind => {
        match callee_kind {
            CalleeKind::Function(expression) => {
                CalleeKind::Function(folder.arena().alloc(folder.fold_expression(expression)))
            }
            CalleeKind::Method(expression, selector) => CalleeKind::Method(
                folder.arena().alloc(folder.fold_expression(expression)),
                folder.fold_member_selector(selector),
            ),
            CalleeKind::NullsafeMethod(expression, selector) => CalleeKind::NullsafeMethod(
                folder.arena().alloc(folder.fold_expression(expression)),
                folder.fold_member_selector(selector),
            ),
            CalleeKind::StaticMethod(expression, selector) => CalleeKind::StaticMethod(
                folder.arena().alloc(folder.fold_expression(expression)),
                folder.fold_member_selector(selector),
            ),
        }
    }

    Access as access => {
        Access { span: access.span, kind: folder.fold_access_kind(&access.kind) }
    }

    AccessKind as access_kind => {
        match access_kind {
            AccessKind::Array(target, index) => AccessKind::Array(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.arena().alloc(folder.fold_expression(index)),
            ),
            AccessKind::Property(target, selector) => AccessKind::Property(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.fold_member_selector(selector),
            ),
            AccessKind::NullsafeProperty(target, selector) => AccessKind::NullsafeProperty(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.fold_member_selector(selector),
            ),
            AccessKind::StaticProperty(target, variable) => AccessKind::StaticProperty(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.fold_variable(variable),
            ),
            AccessKind::ClassConstant(target, selector) => AccessKind::ClassConstant(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.fold_constant_selector(selector),
            ),
        }
    }

    MemberSelector as member_selector => {
        MemberSelector { span: member_selector.span, kind: folder.fold_member_selector_kind(&member_selector.kind) }
    }

    MemberSelectorKind as member_selector_kind => {
        match member_selector_kind {
            MemberSelectorKind::Missing => MemberSelectorKind::Missing,
            MemberSelectorKind::Name(name) => MemberSelectorKind::Name(name.copy_into(folder.arena())),
            MemberSelectorKind::Variable(variable) => {
                MemberSelectorKind::Variable(variable.copy_into(folder.arena()))
            }
            MemberSelectorKind::Expression(expression) => {
                MemberSelectorKind::Expression(folder.arena().alloc(folder.fold_expression(expression)))
            }
        }
    }

    ConstantSelector as constant_selector => {
        ConstantSelector {
            span: constant_selector.span,
            kind: folder.fold_constant_selector_kind(&constant_selector.kind),
        }
    }

    ConstantSelectorKind as constant_selector_kind => {
        match constant_selector_kind {
            ConstantSelectorKind::Missing => ConstantSelectorKind::Missing,
            ConstantSelectorKind::Name(name) => ConstantSelectorKind::Name(name.copy_into(folder.arena())),
            ConstantSelectorKind::Expression(expression) => {
                ConstantSelectorKind::Expression(folder.arena().alloc(folder.fold_expression(expression)))
            }
        }
    }

    Yield as yield_expression => {
        Yield { span: yield_expression.span, kind: folder.fold_yield_kind(&yield_expression.kind) }
    }

    YieldKind as yield_kind => {
        match yield_kind {
            YieldKind::Nothing => YieldKind::Nothing,
            YieldKind::Expression(value) => YieldKind::Expression(folder.arena().alloc(folder.fold_expression(value))),
            YieldKind::Pair(key, value) => YieldKind::Pair(
                folder.arena().alloc(folder.fold_expression(key)),
                folder.arena().alloc(folder.fold_expression(value)),
            ),
            YieldKind::From(value) => YieldKind::From(folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    Match as match_expression => {
        Match {
            span: match_expression.span,
            subject: folder.arena().alloc(folder.fold_expression(match_expression.subject)),
            arms: fold_delimited(folder.arena(), &match_expression.arms, |arm| folder.fold_match_arm(arm)),
        }
    }

    MatchArm as match_arm => {
        MatchArm { span: match_arm.span, kind: folder.fold_match_arm_kind(&match_arm.kind) }
    }

    MatchArmKind as match_arm_kind => {
        match match_arm_kind {
            MatchArmKind::Expression(conditions, body) => MatchArmKind::Expression(
                folder.arena().alloc_slice_fill_iter(conditions.iter().map(|condition| folder.fold_expression(condition))),
                folder.arena().alloc(folder.fold_expression(body)),
            ),
            MatchArmKind::Default(body) => MatchArmKind::Default(folder.arena().alloc(folder.fold_expression(body))),
        }
    }

    ItemExpressionKind as item_expression_kind => {
        match item_expression_kind {
            ItemExpressionKind::AnonymousClass(node) => {
                ItemExpressionKind::AnonymousClass(folder.arena().alloc(folder.fold_anonymous_class(node)))
            }
            ItemExpressionKind::ArrowFunction(node) => {
                ItemExpressionKind::ArrowFunction(folder.arena().alloc(folder.fold_arrow_function(node)))
            }
            ItemExpressionKind::Closure(node) => {
                ItemExpressionKind::Closure(folder.arena().alloc(folder.fold_closure(node)))
            }
        }
    }

    AnonymousClass as anonymous_class => {
        AnonymousClass {
            span: anonymous_class.span,
            modifiers: folder.arena().alloc_slice_copy(anonymous_class.modifiers),
            name: folder.arena().alloc_slice_copy(anonymous_class.name),
            annotation: anonymous_class.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder.arena().alloc_slice_fill_iter(
                anonymous_class.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            arguments: anonymous_class
                .arguments
                .as_ref()
                .map(|arguments| fold_delimited(folder.arena(), arguments, |argument| folder.fold_partial_argument(argument))),
            extends: anonymous_class.extends.map(|extends| copy_ref_into(extends, folder.arena())),
            implements: anonymous_class.implements.map(|implements| copy_ref_into(implements, folder.arena())),
            members: fold_delimited(folder.arena(), &anonymous_class.members, |member| folder.fold_member_item(member)),
        }
    }

    Closure as closure => {
        Closure {
            span: closure.span,
            name: folder.arena().alloc_slice_copy(closure.name),
            annotation: closure.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(closure.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            flags: closure.flags,
            parameters: fold_delimited(folder.arena(), &closure.parameters, |parameter| folder.fold_parameter(parameter)),
            return_type: closure.return_type.map(|return_type| copy_ref_into(return_type, folder.arena())),
            use_variables: closure.use_variables.map(|use_variables| use_variables.copy_into(folder.arena())),
            body: folder.arena().alloc(folder.fold_block(closure.body)),
            direct_accessed_globals: copy_slice_into(closure.direct_accessed_globals, folder.arena()),
        }
    }

    ArrowFunction as arrow_function => {
        ArrowFunction {
            span: arrow_function.span,
            name: folder.arena().alloc_slice_copy(arrow_function.name),
            annotation: arrow_function.annotation.map(|annotation| &*folder.arena().alloc(folder.fold_item_annotation(annotation))),
            attributes: folder.arena().alloc_slice_fill_iter(
                arrow_function.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            flags: arrow_function.flags,
            parameters: fold_delimited(folder.arena(), &arrow_function.parameters, |parameter| {
                folder.fold_parameter(parameter)
            }),
            return_type: arrow_function.return_type.map(|return_type| copy_ref_into(return_type, folder.arena())),
            expression: folder.arena().alloc(folder.fold_expression(arrow_function.expression)),
        }
    }

    Variable as variable => {
        match variable {
            Variable::Direct(direct) => Variable::Direct(direct.copy_into(folder.arena())),
            Variable::Indirect(expression) => {
                Variable::Indirect(folder.arena().alloc(folder.fold_expression(expression)))
            }
            Variable::Nested(nested) => Variable::Nested(folder.arena().alloc(folder.fold_variable(nested))),
        }
    }
}

#[cfg(test)]
mod tests {
    use mago_allocator::LocalArena;

    use mago_span::Span;

    use super::Fold;
    use super::StatementFold;
    use crate::ir::IR;
    use crate::ir::expression::ExpressionKind;
    use crate::ir::item::expression::ItemExpressionKind;
    use crate::ir::item::member::MemberItemKind;
    use crate::ir::item::statement::ItemStatementKind;
    use crate::ir::statement::StatementKind;

    struct Widen<'arena> {
        arena: &'arena LocalArena,
    }

    impl<'arena> Fold<'_, 'arena, LocalArena> for Widen<'arena> {
        type FromItem = String;
        type FromStatement = String;
        type FromExpression = String;
        type ToItem = ();
        type ToStatement = Vec<String>;
        type ToExpression = u32;

        fn arena(&self) -> &'arena LocalArena {
            self.arena
        }

        fn fold_statement_meta(
            &self,
            _span: Span,
            _kind: &StatementKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
        ) -> Self::ToStatement {
            Vec::new()
        }

        fn fold_expression_meta(
            &self,
            _span: Span,
            _kind: &ExpressionKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
        ) -> Self::ToExpression {
            0
        }

        fn fold_item_statement_meta(
            &self,
            _span: Span,
            _kind: &ItemStatementKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
        ) -> Self::ToItem {
        }

        fn fold_item_expression_meta(
            &self,
            _span: Span,
            _kind: &ItemExpressionKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
        ) -> Self::ToItem {
        }

        fn fold_member_item_meta(
            &self,
            _span: Span,
            _kind: &MemberItemKind<'arena, Self::ToItem, Self::ToStatement, Self::ToExpression>,
        ) -> Self::ToItem {
        }
    }

    #[test]
    fn folds_between_distinct_non_copy_meta() {
        let arena = LocalArena::new();
        let widen = Widen { arena: &arena };

        let input: IR<'_, String, String, String> =
            IR { span: Span::zero(), comments: &[], statements: &[], errors: &[] };
        let output: IR<'_, (), Vec<String>, u32> = widen.fold_ir(&input);

        assert!(output.statements.is_empty());
    }

    struct WidenStatements<'arena> {
        arena: &'arena LocalArena,
    }

    impl<'arena> StatementFold<'_, 'arena, LocalArena, u8, u16> for WidenStatements<'arena> {
        type FromStatement = String;
        type ToStatement = Vec<String>;

        fn arena(&self) -> &'arena LocalArena {
            self.arena
        }

        fn fold_statement_meta(
            &self,
            _span: Span,
            _kind: &StatementKind<'arena, u8, Self::ToStatement, u16>,
        ) -> Self::ToStatement {
            Vec::new()
        }
    }

    #[test]
    fn statement_fold_changes_only_the_statement_hole() {
        let arena = LocalArena::new();
        let folder = WidenStatements { arena: &arena };

        let input: IR<'_, u8, String, u16> = IR { span: Span::zero(), comments: &[], statements: &[], errors: &[] };
        let output: IR<'_, u8, Vec<String>, u16> = folder.fold_ir(&input);

        assert!(output.statements.is_empty());
    }
}
