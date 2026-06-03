use bumpalo::Bump;

use mago_span::Span;

use crate::ir::IR;
use crate::ir::argument::Argument;
use crate::ir::argument::PartialArgument;
use crate::ir::attribute::Attribute;
use crate::ir::expression::Access;
use crate::ir::expression::ArrayElement;
use crate::ir::expression::Assignment;
use crate::ir::expression::Binary;
use crate::ir::expression::Call;
use crate::ir::expression::Callee;
use crate::ir::expression::CompositeStringPart;
use crate::ir::expression::Conditional;
use crate::ir::expression::Expression;
use crate::ir::expression::ExpressionKind;
use crate::ir::expression::Instantiation;
use crate::ir::expression::Match;
use crate::ir::expression::MatchArm;
use crate::ir::expression::PartialApplication;
use crate::ir::expression::UnaryPostfix;
use crate::ir::expression::UnaryPrefix;
use crate::ir::expression::Yield;
use crate::ir::expression::annotation::Annotation;
use crate::ir::expression::definition::AnonymousClass;
use crate::ir::expression::definition::ArrowFunction;
use crate::ir::expression::definition::Closure;
use crate::ir::expression::definition::ClosureUseClauseVariable;
use crate::ir::expression::definition::DefinitionExpression;
use crate::ir::expression::definition::DefinitionExpressionKind;
use crate::ir::expression::selector::ConstantSelector;
use crate::ir::expression::selector::MemberSelector;
use crate::ir::hook::Hook;
use crate::ir::hook::HookBody;
use crate::ir::member::ClassLikeConstant;
use crate::ir::member::ClassLikeConstantItem;
use crate::ir::member::EnumCase;
use crate::ir::member::HookedProperty;
use crate::ir::member::Method;
use crate::ir::member::Property;
use crate::ir::member::PropertyItem;
use crate::ir::member::annotation::MethodAnnotation;
use crate::ir::parameter::Parameter;
use crate::ir::parameter::annotation::ParameterAnnotation;
use crate::ir::statement::Declare;
use crate::ir::statement::DeclareItem;
use crate::ir::statement::DoWhile;
use crate::ir::statement::For;
use crate::ir::statement::Foreach;
use crate::ir::statement::GlobalItem;
use crate::ir::statement::If;
use crate::ir::statement::Namespace;
use crate::ir::statement::Statement;
use crate::ir::statement::StatementKind;
use crate::ir::statement::StaticItem;
use crate::ir::statement::Switch;
use crate::ir::statement::SwitchCase;
use crate::ir::statement::Try;
use crate::ir::statement::TryCatchClause;
use crate::ir::statement::While;
use crate::ir::statement::definition::Class;
use crate::ir::statement::definition::Constant;
use crate::ir::statement::definition::ConstantItem;
use crate::ir::statement::definition::DefinitionStatement;
use crate::ir::statement::definition::DefinitionStatementKind;
use crate::ir::statement::definition::Enum;
use crate::ir::statement::definition::Function;
use crate::ir::statement::definition::Interface;
use crate::ir::statement::definition::Trait;
use crate::ir::variable::Variable;

macro_rules! gen_fold_method {
    (mut, ($($from:tt)*), ($($to:tt)*), $node:ident $name:ident $folder:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<fold_ $name>](&mut self, $name: &$node<'arena, $($from)*>) -> $node<'arena, $($to)*> {
                let $folder = self;
                $body
            }
        }
    };
    (shared, ($($from:tt)*), ($($to:tt)*), $node:ident $name:ident $folder:ident $body:block) => {
        paste::paste! {
            #[inline]
            fn [<fold_ $name>](&self, $name: &$node<'arena, $($from)*>) -> $node<'arena, $($to)*> {
                let $folder = self;
                $body
            }
        }
    };
}

macro_rules! gen_fold_all_mut {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(mut, (Self::FromStatement, Self::FromDefinition, Self::FromExpression), (Self::ToStatement, Self::ToDefinition, Self::ToExpression), $node $name $folder $body);
    };
}
macro_rules! gen_fold_all_const {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(shared, (Self::FromStatement, Self::FromDefinition, Self::FromExpression), (Self::ToStatement, Self::ToDefinition, Self::ToExpression), $node $name $folder $body);
    };
}
macro_rules! gen_fold_statement_mut {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(mut, (Self::FromStatement, D, E), (Self::ToStatement, D, E), $node $name $folder $body);
    };
}
macro_rules! gen_fold_statement_const {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(shared, (Self::FromStatement, D, E), (Self::ToStatement, D, E), $node $name $folder $body);
    };
}
macro_rules! gen_fold_expression_mut {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(mut, (S, D, Self::FromExpression), (S, D, Self::ToExpression), $node $name $folder $body);
    };
}
macro_rules! gen_fold_expression_const {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(shared, (S, D, Self::FromExpression), (S, D, Self::ToExpression), $node $name $folder $body);
    };
}
macro_rules! gen_fold_definition_mut {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(mut, (S, Self::FromDefinition, E), (S, Self::ToDefinition, E), $node $name $folder $body);
    };
}
macro_rules! gen_fold_definition_const {
    ($node:ident $name:ident $folder:ident $body:block) => {
        gen_fold_method!(shared, (S, Self::FromDefinition, E), (S, Self::ToDefinition, E), $node $name $folder $body);
    };
}

macro_rules! generate_fold {
    (
        using($folder:ident):
        $( $node:ident as $name:ident => $body:block )*
    ) => {
        /// Folds the [`IR`] with mutable access, transforming all three meta holes at once.
        pub trait MutFold<'arena> {
            type FromStatement;
            type FromDefinition;
            type FromExpression;
            type ToStatement;
            type ToDefinition;
            type ToExpression;

            fn arena(&self) -> &'arena Bump;

            fn fold_statement_meta(&mut self, span: Span, kind: &StatementKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>) -> Self::ToStatement;
            fn fold_expression_meta(&mut self, span: Span, kind: &ExpressionKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>) -> Self::ToExpression;
            fn fold_definition_statement_meta(&mut self, kind: &DefinitionStatementKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>) -> Self::ToDefinition;
            fn fold_definition_expression_meta(&mut self, kind: &DefinitionExpressionKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>) -> Self::ToDefinition;

            #[inline]
            fn fold_statement(&mut self, statement: &Statement<'arena, Self::FromStatement, Self::FromDefinition, Self::FromExpression>) -> Statement<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression> {
                let kind = self.fold_statement_kind(&statement.kind);
                let meta = self.fold_statement_meta(statement.span, &kind);
                Statement { meta, span: statement.span, kind }
            }
            #[inline]
            fn fold_expression(&mut self, expression: &Expression<'arena, Self::FromStatement, Self::FromDefinition, Self::FromExpression>) -> Expression<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression> {
                let kind = self.fold_expression_kind(&expression.kind);
                let meta = self.fold_expression_meta(expression.span, &kind);
                Expression { meta, span: expression.span, kind }
            }
            #[inline]
            fn fold_definition_statement(&mut self, definition: &DefinitionStatement<'arena, Self::FromStatement, Self::FromDefinition, Self::FromExpression>) -> DefinitionStatement<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression> {
                let kind = self.fold_definition_statement_kind(&definition.kind);
                let meta = self.fold_definition_statement_meta(&kind);
                DefinitionStatement { meta, kind }
            }
            #[inline]
            fn fold_definition_expression(&mut self, definition: &DefinitionExpression<'arena, Self::FromStatement, Self::FromDefinition, Self::FromExpression>) -> DefinitionExpression<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression> {
                let kind = self.fold_definition_expression_kind(&definition.kind);
                let meta = self.fold_definition_expression_meta(&kind);
                DefinitionExpression { meta, kind }
            }

            $( gen_fold_all_mut!($node $name $folder $body); )*
        }

        /// Folds the [`IR`] with shared access, transforming all three meta holes at once.
        pub trait Fold<'arena> {
            type FromStatement;
            type FromDefinition;
            type FromExpression;
            type ToStatement;
            type ToDefinition;
            type ToExpression;

            fn arena(&self) -> &'arena Bump;

            fn fold_statement_meta(&self, span: Span, kind: &StatementKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>) -> Self::ToStatement;
            fn fold_expression_meta(&self, span: Span, kind: &ExpressionKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>) -> Self::ToExpression;
            fn fold_definition_statement_meta(&self, kind: &DefinitionStatementKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>) -> Self::ToDefinition;
            fn fold_definition_expression_meta(&self, kind: &DefinitionExpressionKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>) -> Self::ToDefinition;

            #[inline]
            fn fold_statement(&self, statement: &Statement<'arena, Self::FromStatement, Self::FromDefinition, Self::FromExpression>) -> Statement<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression> {
                let kind = self.fold_statement_kind(&statement.kind);
                let meta = self.fold_statement_meta(statement.span, &kind);
                Statement { meta, span: statement.span, kind }
            }
            #[inline]
            fn fold_expression(&self, expression: &Expression<'arena, Self::FromStatement, Self::FromDefinition, Self::FromExpression>) -> Expression<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression> {
                let kind = self.fold_expression_kind(&expression.kind);
                let meta = self.fold_expression_meta(expression.span, &kind);
                Expression { meta, span: expression.span, kind }
            }
            #[inline]
            fn fold_definition_statement(&self, definition: &DefinitionStatement<'arena, Self::FromStatement, Self::FromDefinition, Self::FromExpression>) -> DefinitionStatement<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression> {
                let kind = self.fold_definition_statement_kind(&definition.kind);
                let meta = self.fold_definition_statement_meta(&kind);
                DefinitionStatement { meta, kind }
            }
            #[inline]
            fn fold_definition_expression(&self, definition: &DefinitionExpression<'arena, Self::FromStatement, Self::FromDefinition, Self::FromExpression>) -> DefinitionExpression<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression> {
                let kind = self.fold_definition_expression_kind(&definition.kind);
                let meta = self.fold_definition_expression_meta(&kind);
                DefinitionExpression { meta, kind }
            }

            $( gen_fold_all_const!($node $name $folder $body); )*
        }

        /// Folds only the statement meta hole, leaving definition and expression metas untouched.
        pub trait MutStatementFold<'arena, D, E>
        where
            D: Copy,
            E: Copy,
        {
            type FromStatement;
            type ToStatement;

            fn arena(&self) -> &'arena Bump;

            fn fold_statement_meta(&mut self, span: Span, kind: &StatementKind<'arena, Self::ToStatement, D, E>) -> Self::ToStatement;

            #[inline]
            fn fold_statement(&mut self, statement: &Statement<'arena, Self::FromStatement, D, E>) -> Statement<'arena, Self::ToStatement, D, E> {
                let kind = self.fold_statement_kind(&statement.kind);
                let meta = self.fold_statement_meta(statement.span, &kind);
                Statement { meta, span: statement.span, kind }
            }
            #[inline]
            fn fold_expression(&mut self, expression: &Expression<'arena, Self::FromStatement, D, E>) -> Expression<'arena, Self::ToStatement, D, E> {
                let kind = self.fold_expression_kind(&expression.kind);
                Expression { meta: expression.meta, span: expression.span, kind }
            }
            #[inline]
            fn fold_definition_statement(&mut self, definition: &DefinitionStatement<'arena, Self::FromStatement, D, E>) -> DefinitionStatement<'arena, Self::ToStatement, D, E> {
                let kind = self.fold_definition_statement_kind(&definition.kind);
                DefinitionStatement { meta: definition.meta, kind }
            }
            #[inline]
            fn fold_definition_expression(&mut self, definition: &DefinitionExpression<'arena, Self::FromStatement, D, E>) -> DefinitionExpression<'arena, Self::ToStatement, D, E> {
                let kind = self.fold_definition_expression_kind(&definition.kind);
                DefinitionExpression { meta: definition.meta, kind }
            }

            $( gen_fold_statement_mut!($node $name $folder $body); )*
        }

        /// Folds only the statement meta hole, leaving definition and expression metas untouched.
        pub trait StatementFold<'arena, D, E>
        where
            D: Copy,
            E: Copy,
        {
            type FromStatement;
            type ToStatement;

            fn arena(&self) -> &'arena Bump;

            fn fold_statement_meta(&self, span: Span, kind: &StatementKind<'arena, Self::ToStatement, D, E>) -> Self::ToStatement;

            #[inline]
            fn fold_statement(&self, statement: &Statement<'arena, Self::FromStatement, D, E>) -> Statement<'arena, Self::ToStatement, D, E> {
                let kind = self.fold_statement_kind(&statement.kind);
                let meta = self.fold_statement_meta(statement.span, &kind);
                Statement { meta, span: statement.span, kind }
            }
            #[inline]
            fn fold_expression(&self, expression: &Expression<'arena, Self::FromStatement, D, E>) -> Expression<'arena, Self::ToStatement, D, E> {
                let kind = self.fold_expression_kind(&expression.kind);
                Expression { meta: expression.meta, span: expression.span, kind }
            }
            #[inline]
            fn fold_definition_statement(&self, definition: &DefinitionStatement<'arena, Self::FromStatement, D, E>) -> DefinitionStatement<'arena, Self::ToStatement, D, E> {
                let kind = self.fold_definition_statement_kind(&definition.kind);
                DefinitionStatement { meta: definition.meta, kind }
            }
            #[inline]
            fn fold_definition_expression(&self, definition: &DefinitionExpression<'arena, Self::FromStatement, D, E>) -> DefinitionExpression<'arena, Self::ToStatement, D, E> {
                let kind = self.fold_definition_expression_kind(&definition.kind);
                DefinitionExpression { meta: definition.meta, kind }
            }

            $( gen_fold_statement_const!($node $name $folder $body); )*
        }

        /// Folds only the expression meta hole, leaving statement and definition metas untouched.
        pub trait MutExpressionFold<'arena, S, D>
        where
            S: Copy,
            D: Copy,
        {
            type FromExpression;
            type ToExpression;

            fn arena(&self) -> &'arena Bump;

            fn fold_expression_meta(&mut self, span: Span, kind: &ExpressionKind<'arena, S, D, Self::ToExpression>) -> Self::ToExpression;

            #[inline]
            fn fold_statement(&mut self, statement: &Statement<'arena, S, D, Self::FromExpression>) -> Statement<'arena, S, D, Self::ToExpression> {
                let kind = self.fold_statement_kind(&statement.kind);
                Statement { meta: statement.meta, span: statement.span, kind }
            }
            #[inline]
            fn fold_expression(&mut self, expression: &Expression<'arena, S, D, Self::FromExpression>) -> Expression<'arena, S, D, Self::ToExpression> {
                let kind = self.fold_expression_kind(&expression.kind);
                let meta = self.fold_expression_meta(expression.span, &kind);
                Expression { meta, span: expression.span, kind }
            }
            #[inline]
            fn fold_definition_statement(&mut self, definition: &DefinitionStatement<'arena, S, D, Self::FromExpression>) -> DefinitionStatement<'arena, S, D, Self::ToExpression> {
                let kind = self.fold_definition_statement_kind(&definition.kind);
                DefinitionStatement { meta: definition.meta, kind }
            }
            #[inline]
            fn fold_definition_expression(&mut self, definition: &DefinitionExpression<'arena, S, D, Self::FromExpression>) -> DefinitionExpression<'arena, S, D, Self::ToExpression> {
                let kind = self.fold_definition_expression_kind(&definition.kind);
                DefinitionExpression { meta: definition.meta, kind }
            }

            $( gen_fold_expression_mut!($node $name $folder $body); )*
        }

        /// Folds only the expression meta hole, leaving statement and definition metas untouched.
        pub trait ExpressionFold<'arena, S, D>
        where
            S: Copy,
            D: Copy,
        {
            type FromExpression;
            type ToExpression;

            fn arena(&self) -> &'arena Bump;

            fn fold_expression_meta(&self, span: Span, kind: &ExpressionKind<'arena, S, D, Self::ToExpression>) -> Self::ToExpression;

            #[inline]
            fn fold_statement(&self, statement: &Statement<'arena, S, D, Self::FromExpression>) -> Statement<'arena, S, D, Self::ToExpression> {
                let kind = self.fold_statement_kind(&statement.kind);
                Statement { meta: statement.meta, span: statement.span, kind }
            }
            #[inline]
            fn fold_expression(&self, expression: &Expression<'arena, S, D, Self::FromExpression>) -> Expression<'arena, S, D, Self::ToExpression> {
                let kind = self.fold_expression_kind(&expression.kind);
                let meta = self.fold_expression_meta(expression.span, &kind);
                Expression { meta, span: expression.span, kind }
            }
            #[inline]
            fn fold_definition_statement(&self, definition: &DefinitionStatement<'arena, S, D, Self::FromExpression>) -> DefinitionStatement<'arena, S, D, Self::ToExpression> {
                let kind = self.fold_definition_statement_kind(&definition.kind);
                DefinitionStatement { meta: definition.meta, kind }
            }
            #[inline]
            fn fold_definition_expression(&self, definition: &DefinitionExpression<'arena, S, D, Self::FromExpression>) -> DefinitionExpression<'arena, S, D, Self::ToExpression> {
                let kind = self.fold_definition_expression_kind(&definition.kind);
                DefinitionExpression { meta: definition.meta, kind }
            }

            $( gen_fold_expression_const!($node $name $folder $body); )*
        }

        /// Folds only the definition meta hole, leaving statement and expression metas untouched.
        pub trait MutDefinitionFold<'arena, S, E>
        where
            S: Copy,
            E: Copy,
        {
            type FromDefinition;
            type ToDefinition;

            fn arena(&self) -> &'arena Bump;

            fn fold_definition_statement_meta(&mut self, kind: &DefinitionStatementKind<'arena, S, Self::ToDefinition, E>) -> Self::ToDefinition;
            fn fold_definition_expression_meta(&mut self, kind: &DefinitionExpressionKind<'arena, S, Self::ToDefinition, E>) -> Self::ToDefinition;

            #[inline]
            fn fold_statement(&mut self, statement: &Statement<'arena, S, Self::FromDefinition, E>) -> Statement<'arena, S, Self::ToDefinition, E> {
                let kind = self.fold_statement_kind(&statement.kind);
                Statement { meta: statement.meta, span: statement.span, kind }
            }
            #[inline]
            fn fold_expression(&mut self, expression: &Expression<'arena, S, Self::FromDefinition, E>) -> Expression<'arena, S, Self::ToDefinition, E> {
                let kind = self.fold_expression_kind(&expression.kind);
                Expression { meta: expression.meta, span: expression.span, kind }
            }
            #[inline]
            fn fold_definition_statement(&mut self, definition: &DefinitionStatement<'arena, S, Self::FromDefinition, E>) -> DefinitionStatement<'arena, S, Self::ToDefinition, E> {
                let kind = self.fold_definition_statement_kind(&definition.kind);
                let meta = self.fold_definition_statement_meta(&kind);
                DefinitionStatement { meta, kind }
            }
            #[inline]
            fn fold_definition_expression(&mut self, definition: &DefinitionExpression<'arena, S, Self::FromDefinition, E>) -> DefinitionExpression<'arena, S, Self::ToDefinition, E> {
                let kind = self.fold_definition_expression_kind(&definition.kind);
                let meta = self.fold_definition_expression_meta(&kind);
                DefinitionExpression { meta, kind }
            }

            $( gen_fold_definition_mut!($node $name $folder $body); )*
        }

        /// Folds only the definition meta hole, leaving statement and expression metas untouched.
        pub trait DefinitionFold<'arena, S, E>
        where
            S: Copy,
            E: Copy,
        {
            type FromDefinition;
            type ToDefinition;

            fn arena(&self) -> &'arena Bump;

            fn fold_definition_statement_meta(&self, kind: &DefinitionStatementKind<'arena, S, Self::ToDefinition, E>) -> Self::ToDefinition;
            fn fold_definition_expression_meta(&self, kind: &DefinitionExpressionKind<'arena, S, Self::ToDefinition, E>) -> Self::ToDefinition;

            #[inline]
            fn fold_statement(&self, statement: &Statement<'arena, S, Self::FromDefinition, E>) -> Statement<'arena, S, Self::ToDefinition, E> {
                let kind = self.fold_statement_kind(&statement.kind);
                Statement { meta: statement.meta, span: statement.span, kind }
            }
            #[inline]
            fn fold_expression(&self, expression: &Expression<'arena, S, Self::FromDefinition, E>) -> Expression<'arena, S, Self::ToDefinition, E> {
                let kind = self.fold_expression_kind(&expression.kind);
                Expression { meta: expression.meta, span: expression.span, kind }
            }
            #[inline]
            fn fold_definition_statement(&self, definition: &DefinitionStatement<'arena, S, Self::FromDefinition, E>) -> DefinitionStatement<'arena, S, Self::ToDefinition, E> {
                let kind = self.fold_definition_statement_kind(&definition.kind);
                let meta = self.fold_definition_statement_meta(&kind);
                DefinitionStatement { meta, kind }
            }
            #[inline]
            fn fold_definition_expression(&self, definition: &DefinitionExpression<'arena, S, Self::FromDefinition, E>) -> DefinitionExpression<'arena, S, Self::ToDefinition, E> {
                let kind = self.fold_definition_expression_kind(&definition.kind);
                let meta = self.fold_definition_expression_meta(&kind);
                DefinitionExpression { meta, kind }
            }

            $( gen_fold_definition_const!($node $name $folder $body); )*
        }
    };
}

generate_fold! {
    using(folder):

    IR as ir => {
        IR {
            statements: folder
                .arena()
                .alloc_slice_fill_iter(ir.statements.iter().map(|statement| folder.fold_statement(statement))),
        }
    }

    StatementKind as statement_kind => {
        match statement_kind {
            StatementKind::Inline(value) => StatementKind::Inline(value),
            StatementKind::Namespace(node) => {
                StatementKind::Namespace(folder.arena().alloc(folder.fold_namespace(node)))
            }
            StatementKind::Sequence(statements) => StatementKind::Sequence(
                folder.arena().alloc_slice_fill_iter(statements.iter().map(|statement| folder.fold_statement(statement))),
            ),
            StatementKind::Definition(node) => {
                StatementKind::Definition(folder.arena().alloc(folder.fold_definition_statement(node)))
            }
            StatementKind::Declare(node) => StatementKind::Declare(folder.arena().alloc(folder.fold_declare(node))),
            StatementKind::Goto(name) => StatementKind::Goto(*name),
            StatementKind::Label(name) => StatementKind::Label(*name),
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
            StatementKind::Global(items) => StatementKind::Global(
                folder.arena().alloc_slice_fill_iter(items.iter().map(|item| folder.fold_global_item(item))),
            ),
            StatementKind::Static(items) => StatementKind::Static(
                folder.arena().alloc_slice_fill_iter(items.iter().map(|item| folder.fold_static_item(item))),
            ),
            StatementKind::Unset(values) => StatementKind::Unset(
                folder.arena().alloc_slice_fill_iter(values.iter().map(|value| folder.fold_expression(value))),
            ),
            StatementKind::VariableBindingAnnotation(node) => StatementKind::VariableBindingAnnotation(node),
            StatementKind::HaltCompiler => StatementKind::HaltCompiler,
            StatementKind::Noop => StatementKind::Noop,
        }
    }

    Namespace as namespace => {
        Namespace { name: namespace.name, statement: folder.arena().alloc(folder.fold_statement(namespace.statement)) }
    }

    Declare as declare => {
        Declare {
            items: folder.arena().alloc_slice_fill_iter(declare.items.iter().map(|item| folder.fold_declare_item(item))),
            statement: folder.arena().alloc(folder.fold_statement(declare.statement)),
        }
    }

    DeclareItem as declare_item => {
        DeclareItem {
            name: declare_item.name,
            value: declare_item.value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    Try as try_statement => {
        Try {
            statement: folder.arena().alloc(folder.fold_statement(try_statement.statement)),
            catch_clauses: folder.arena().alloc_slice_fill_iter(
                try_statement.catch_clauses.iter().map(|clause| folder.fold_try_catch_clause(clause)),
            ),
            finally_clause: try_statement
                .finally_clause
                .map(|clause| &*folder.arena().alloc(folder.fold_statement(clause))),
        }
    }

    TryCatchClause as try_catch_clause => {
        TryCatchClause {
            r#type: try_catch_clause.r#type,
            variable: try_catch_clause.variable,
            statement: folder.arena().alloc(folder.fold_statement(try_catch_clause.statement)),
        }
    }

    Foreach as foreach => {
        Foreach {
            expression: folder.arena().alloc(folder.fold_expression(foreach.expression)),
            key: foreach.key.map(|key| &*folder.arena().alloc(folder.fold_expression(key))),
            value: folder.arena().alloc(folder.fold_expression(foreach.value)),
            statement: folder.arena().alloc(folder.fold_statement(foreach.statement)),
        }
    }

    For as for_loop => {
        For {
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
            condition: folder.arena().alloc(folder.fold_expression(while_loop.condition)),
            statement: folder.arena().alloc(folder.fold_statement(while_loop.statement)),
        }
    }

    DoWhile as do_while => {
        DoWhile {
            statement: folder.arena().alloc(folder.fold_statement(do_while.statement)),
            condition: folder.arena().alloc(folder.fold_expression(do_while.condition)),
        }
    }

    Switch as switch => {
        Switch {
            subject: folder.arena().alloc(folder.fold_expression(switch.subject)),
            cases: folder.arena().alloc_slice_fill_iter(switch.cases.iter().map(|case| folder.fold_switch_case(case))),
        }
    }

    SwitchCase as switch_case => {
        match switch_case {
            SwitchCase::Expression(expression, statement) => SwitchCase::Expression(
                folder.arena().alloc(folder.fold_expression(expression)),
                folder.arena().alloc(folder.fold_statement(statement)),
            ),
            SwitchCase::Default(statement) => SwitchCase::Default(folder.arena().alloc(folder.fold_statement(statement))),
        }
    }

    If as if_statement => {
        If {
            condition: folder.arena().alloc(folder.fold_expression(if_statement.condition)),
            then: folder.arena().alloc(folder.fold_statement(if_statement.then)),
            r#else: if_statement.r#else.map(|statement| &*folder.arena().alloc(folder.fold_statement(statement))),
        }
    }

    StaticItem as static_item => {
        StaticItem {
            variable: static_item.variable,
            type_annotation: static_item.type_annotation,
            value: static_item.value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    GlobalItem as global_item => {
        GlobalItem { variable: folder.fold_variable(&global_item.variable), type_annotation: global_item.type_annotation }
    }

    DefinitionStatementKind as definition_statement_kind => {
        match definition_statement_kind {
            DefinitionStatementKind::Class(node) => {
                DefinitionStatementKind::Class(folder.arena().alloc(folder.fold_class(node)))
            }
            DefinitionStatementKind::Interface(node) => {
                DefinitionStatementKind::Interface(folder.arena().alloc(folder.fold_interface(node)))
            }
            DefinitionStatementKind::Trait(node) => {
                DefinitionStatementKind::Trait(folder.arena().alloc(folder.fold_trait_definition(node)))
            }
            DefinitionStatementKind::Enum(node) => {
                DefinitionStatementKind::Enum(folder.arena().alloc(folder.fold_enum_definition(node)))
            }
            DefinitionStatementKind::Constant(node) => {
                DefinitionStatementKind::Constant(folder.arena().alloc(folder.fold_constant(node)))
            }
            DefinitionStatementKind::Function(node) => {
                DefinitionStatementKind::Function(folder.arena().alloc(folder.fold_function(node)))
            }
        }
    }

    Class as class => {
        Class {
            flags: class.flags,
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(class.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            name: class.name,
            type_parameter_annotations: class.type_parameter_annotations,
            modifiers: class.modifiers,
            type_alias_annotations: class.type_alias_annotations,
            imported_type_alias_annotations: class.imported_type_alias_annotations,
            extends: class.extends,
            extends_annotations: class.extends_annotations,
            implements: class.implements,
            implements_annotations: class.implements_annotations,
            sealed_annotation: class.sealed_annotation,
            mixin_annotations: class.mixin_annotations,
            trait_uses: class.trait_uses,
            constants: folder
                .arena()
                .alloc_slice_fill_iter(class.constants.iter().map(|constant| folder.fold_class_like_constant(constant))),
            properties: folder
                .arena()
                .alloc_slice_fill_iter(class.properties.iter().map(|property| folder.fold_property(property))),
            hooked_properties: folder.arena().alloc_slice_fill_iter(
                class.hooked_properties.iter().map(|property| folder.fold_hooked_property(property)),
            ),
            property_annotations: class.property_annotations,
            methods: folder
                .arena()
                .alloc_slice_fill_iter(class.methods.iter().map(|method| folder.fold_method(method))),
            method_annotations: folder.arena().alloc_slice_fill_iter(
                class.method_annotations.iter().map(|annotation| folder.fold_method_annotation(annotation)),
            ),
        }
    }

    Interface as interface => {
        Interface {
            flags: interface.flags,
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(interface.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            name: interface.name,
            type_parameter_annotations: interface.type_parameter_annotations,
            type_alias_annotations: interface.type_alias_annotations,
            imported_type_alias_annotations: interface.imported_type_alias_annotations,
            extends: interface.extends,
            extends_annotations: interface.extends_annotations,
            sealed_annotation: interface.sealed_annotation,
            mixin_annotations: interface.mixin_annotations,
            constants: folder.arena().alloc_slice_fill_iter(
                interface.constants.iter().map(|constant| folder.fold_class_like_constant(constant)),
            ),
            hooked_properties: folder.arena().alloc_slice_fill_iter(
                interface.hooked_properties.iter().map(|property| folder.fold_hooked_property(property)),
            ),
            methods: folder
                .arena()
                .alloc_slice_fill_iter(interface.methods.iter().map(|method| folder.fold_method(method))),
            method_annotations: folder.arena().alloc_slice_fill_iter(
                interface.method_annotations.iter().map(|annotation| folder.fold_method_annotation(annotation)),
            ),
        }
    }

    Trait as trait_definition => {
        Trait {
            flags: trait_definition.flags,
            attributes: folder.arena().alloc_slice_fill_iter(
                trait_definition.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            name: trait_definition.name,
            type_parameter_annotations: trait_definition.type_parameter_annotations,
            type_alias_annotations: trait_definition.type_alias_annotations,
            imported_type_alias_annotations: trait_definition.imported_type_alias_annotations,
            require_extends_annotations: trait_definition.require_extends_annotations,
            require_implements_annotations: trait_definition.require_implements_annotations,
            trait_uses: trait_definition.trait_uses,
            constants: folder.arena().alloc_slice_fill_iter(
                trait_definition.constants.iter().map(|constant| folder.fold_class_like_constant(constant)),
            ),
            properties: folder.arena().alloc_slice_fill_iter(
                trait_definition.properties.iter().map(|property| folder.fold_property(property)),
            ),
            hooked_properties: folder.arena().alloc_slice_fill_iter(
                trait_definition.hooked_properties.iter().map(|property| folder.fold_hooked_property(property)),
            ),
            property_annotations: trait_definition.property_annotations,
            methods: folder
                .arena()
                .alloc_slice_fill_iter(trait_definition.methods.iter().map(|method| folder.fold_method(method))),
            method_annotations: folder.arena().alloc_slice_fill_iter(
                trait_definition.method_annotations.iter().map(|annotation| folder.fold_method_annotation(annotation)),
            ),
        }
    }

    Enum as enum_definition => {
        Enum {
            flags: enum_definition.flags,
            attributes: folder.arena().alloc_slice_fill_iter(
                enum_definition.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            name: enum_definition.name,
            backing_type: enum_definition.backing_type,
            type_alias_annotations: enum_definition.type_alias_annotations,
            imported_type_alias_annotations: enum_definition.imported_type_alias_annotations,
            implements: enum_definition.implements,
            implements_annotations: enum_definition.implements_annotations,
            trait_uses: enum_definition.trait_uses,
            constants: folder.arena().alloc_slice_fill_iter(
                enum_definition.constants.iter().map(|constant| folder.fold_class_like_constant(constant)),
            ),
            enum_cases: folder
                .arena()
                .alloc_slice_fill_iter(enum_definition.enum_cases.iter().map(|case| folder.fold_enum_case(case))),
            methods: folder
                .arena()
                .alloc_slice_fill_iter(enum_definition.methods.iter().map(|method| folder.fold_method(method))),
            method_annotations: folder.arena().alloc_slice_fill_iter(
                enum_definition.method_annotations.iter().map(|annotation| folder.fold_method_annotation(annotation)),
            ),
        }
    }

    Constant as constant => {
        Constant {
            flags: constant.flags,
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(constant.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            type_annotation: constant.type_annotation,
            items: folder.arena().alloc_slice_fill_iter(constant.items.iter().map(|item| folder.fold_constant_item(item))),
        }
    }

    ConstantItem as constant_item => {
        ConstantItem { name: constant_item.name, value: folder.arena().alloc(folder.fold_expression(constant_item.value)) }
    }

    Function as function => {
        Function {
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(function.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            flags: function.flags,
            name: function.name,
            type_parameter_annotations: function.type_parameter_annotations,
            parameters: folder
                .arena()
                .alloc_slice_fill_iter(function.parameters.iter().map(|parameter| folder.fold_parameter(parameter))),
            where_constraint_annotations: function.where_constraint_annotations,
            return_by_reference: function.return_by_reference,
            return_type: function.return_type,
            return_type_annotation: function.return_type_annotation,
            throws_annotations: function.throws_annotations,
            assert_annotations: function.assert_annotations,
            assert_if_true_annotations: function.assert_if_true_annotations,
            assert_if_false_annotations: function.assert_if_false_annotations,
            body: folder.arena().alloc(folder.fold_statement(function.body)),
        }
    }

    Method as method => {
        Method {
            span: method.span,
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(method.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            flags: method.flags,
            modifiers: method.modifiers,
            name: method.name,
            type_parameter_annotations: method.type_parameter_annotations,
            parameters: folder
                .arena()
                .alloc_slice_fill_iter(method.parameters.iter().map(|parameter| folder.fold_parameter(parameter))),
            where_constraint_annotations: method.where_constraint_annotations,
            return_by_reference: method.return_by_reference,
            return_type: method.return_type,
            return_type_annotation: method.return_type_annotation,
            throws: method.throws,
            asserts: method.asserts,
            asserts_if_true: method.asserts_if_true,
            asserts_if_false: method.asserts_if_false,
            self_out_annotation: method.self_out_annotation,
            body: method.body.map(|body| &*folder.arena().alloc(folder.fold_statement(body))),
        }
    }

    Property as property => {
        Property {
            span: property.span,
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(property.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            flags: property.flags,
            modifiers: property.modifiers,
            r#type: property.r#type,
            type_annotation: property.type_annotation,
            items: folder.arena().alloc_slice_fill_iter(property.items.iter().map(|item| folder.fold_property_item(item))),
        }
    }

    PropertyItem as property_item => {
        PropertyItem {
            variable: property_item.variable,
            default_value: property_item.default_value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    HookedProperty as hooked_property => {
        HookedProperty {
            span: hooked_property.span,
            attributes: folder.arena().alloc_slice_fill_iter(
                hooked_property.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            flags: hooked_property.flags,
            modifiers: hooked_property.modifiers,
            r#type: hooked_property.r#type,
            type_annotation: hooked_property.type_annotation,
            item: folder.fold_property_item(&hooked_property.item),
            hooks: folder.arena().alloc_slice_fill_iter(hooked_property.hooks.iter().map(|hook| folder.fold_hook(hook))),
        }
    }

    ClassLikeConstant as class_like_constant => {
        ClassLikeConstant {
            span: class_like_constant.span,
            attributes: folder.arena().alloc_slice_fill_iter(
                class_like_constant.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            modifiers: class_like_constant.modifiers,
            r#type: class_like_constant.r#type,
            type_annotation: class_like_constant.type_annotation,
            items: folder.arena().alloc_slice_fill_iter(
                class_like_constant.items.iter().map(|item| folder.fold_class_like_constant_item(item)),
            ),
        }
    }

    ClassLikeConstantItem as class_like_constant_item => {
        ClassLikeConstantItem {
            name: class_like_constant_item.name,
            value: folder.arena().alloc(folder.fold_expression(class_like_constant_item.value)),
        }
    }

    EnumCase as enum_case => {
        EnumCase {
            span: enum_case.span,
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(enum_case.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            name: enum_case.name,
            value: enum_case.value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    Hook as hook => {
        Hook {
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(hook.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            modifiers: hook.modifiers,
            return_by_reference: hook.return_by_reference,
            name: hook.name,
            is_variadic: hook.is_variadic,
            parameters: folder
                .arena()
                .alloc_slice_fill_iter(hook.parameters.iter().map(|parameter| folder.fold_parameter(parameter))),
            body: hook.body.as_ref().map(|body| match body {
                HookBody::Expression(expression) => {
                    HookBody::Expression(folder.arena().alloc(folder.fold_expression(expression)))
                }
                HookBody::Statements(statements) => HookBody::Statements(
                    folder.arena().alloc_slice_fill_iter(statements.iter().map(|statement| folder.fold_statement(statement))),
                ),
            }),
        }
    }

    Parameter as parameter => {
        Parameter {
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(parameter.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            modifiers: parameter.modifiers,
            r#type: parameter.r#type,
            type_annotation: parameter.type_annotation,
            out_annotation: parameter.out_annotation,
            is_by_reference: parameter.is_by_reference,
            is_variadic: parameter.is_variadic,
            variable: parameter.variable,
            default_value: parameter.default_value.map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
            hooks: folder.arena().alloc_slice_fill_iter(parameter.hooks.iter().map(|hook| folder.fold_hook(hook))),
        }
    }

    MethodAnnotation as method_annotation => {
        MethodAnnotation {
            span: method_annotation.span,
            r#static: method_annotation.r#static,
            name: method_annotation.name,
            type_parameters: method_annotation.type_parameters,
            parameters: folder.arena().alloc_slice_fill_iter(
                method_annotation.parameters.iter().map(|parameter| folder.fold_parameter_annotation(parameter)),
            ),
            return_type: method_annotation.return_type,
        }
    }

    ParameterAnnotation as parameter_annotation => {
        ParameterAnnotation {
            r#type: parameter_annotation.r#type,
            is_by_reference: parameter_annotation.is_by_reference,
            is_variadic: parameter_annotation.is_variadic,
            variable: parameter_annotation.variable,
            default_value: parameter_annotation
                .default_value
                .map(|value| &*folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    Attribute as attribute => {
        Attribute {
            span: attribute.span,
            class: attribute.class,
            arguments: folder
                .arena()
                .alloc_slice_fill_iter(attribute.arguments.iter().map(|argument| folder.fold_argument(argument))),
        }
    }

    Argument as argument => {
        match argument {
            Argument::Value(expression) => Argument::Value(folder.arena().alloc(folder.fold_expression(expression))),
            Argument::Variadic(expression) => {
                Argument::Variadic(folder.arena().alloc(folder.fold_expression(expression)))
            }
            Argument::Named(name, expression) => {
                Argument::Named(*name, folder.arena().alloc(folder.fold_expression(expression)))
            }
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
            PartialArgument::Named(name, expression) => {
                PartialArgument::Named(*name, folder.arena().alloc(folder.fold_expression(expression)))
            }
            PartialArgument::Placeholder => PartialArgument::Placeholder,
            PartialArgument::NamedPlaceholder(name) => PartialArgument::NamedPlaceholder(*name),
            PartialArgument::VariadicPlaceholder => PartialArgument::VariadicPlaceholder,
        }
    }

    ExpressionKind as expression_kind => {
        match expression_kind {
            ExpressionKind::Binary(node) => ExpressionKind::Binary(folder.arena().alloc(folder.fold_binary(node))),
            ExpressionKind::UnaryPrefix(node) => {
                ExpressionKind::UnaryPrefix(folder.arena().alloc(folder.fold_unary_prefix(node)))
            }
            ExpressionKind::UnaryPostfix(node) => {
                ExpressionKind::UnaryPostfix(folder.arena().alloc(folder.fold_unary_postfix(node)))
            }
            ExpressionKind::Literal(node) => ExpressionKind::Literal(node),
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
            ExpressionKind::Array(elements) => ExpressionKind::Array(
                folder.arena().alloc_slice_fill_iter(elements.iter().map(|element| folder.fold_array_element(element))),
            ),
            ExpressionKind::List(elements) => ExpressionKind::List(
                folder.arena().alloc_slice_fill_iter(elements.iter().map(|element| folder.fold_array_element(element))),
            ),
            ExpressionKind::ArrayAppend(node) => {
                ExpressionKind::ArrayAppend(folder.arena().alloc(folder.fold_expression(node)))
            }
            ExpressionKind::Definition(node) => {
                ExpressionKind::Definition(folder.arena().alloc(folder.fold_definition_expression(node)))
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
            ExpressionKind::Isset(values) => ExpressionKind::Isset(
                folder.arena().alloc_slice_fill_iter(values.iter().map(|value| folder.fold_expression(value))),
            ),
            ExpressionKind::Exit(arguments) => ExpressionKind::Exit(
                folder.arena().alloc_slice_fill_iter(arguments.iter().map(|argument| folder.fold_argument(argument))),
            ),
            ExpressionKind::MagicConstant(value) => ExpressionKind::MagicConstant(*value),
            ExpressionKind::Constant(identifier) => ExpressionKind::Constant(*identifier),
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
            ExpressionKind::Identifier(identifier) => ExpressionKind::Identifier(*identifier),
            ExpressionKind::Parent => ExpressionKind::Parent,
            ExpressionKind::Self_ => ExpressionKind::Self_,
            ExpressionKind::Static => ExpressionKind::Static,
            ExpressionKind::SyntaxError => ExpressionKind::SyntaxError,
        }
    }

    Assignment as assignment => {
        Assignment {
            left: folder.arena().alloc(folder.fold_expression(assignment.left)),
            operator: assignment.operator,
            right: folder.arena().alloc(folder.fold_expression(assignment.right)),
        }
    }

    Annotation as annotation => {
        Annotation {
            expression: folder.arena().alloc(folder.fold_expression(annotation.expression)),
            type_annotation: annotation.type_annotation,
        }
    }

    Binary as binary => {
        Binary {
            left: folder.arena().alloc(folder.fold_expression(binary.left)),
            operator: binary.operator,
            right: folder.arena().alloc(folder.fold_expression(binary.right)),
        }
    }

    UnaryPrefix as unary_prefix => {
        UnaryPrefix {
            operator: unary_prefix.operator,
            operand: folder.arena().alloc(folder.fold_expression(unary_prefix.operand)),
        }
    }

    UnaryPostfix as unary_postfix => {
        UnaryPostfix {
            operand: folder.arena().alloc(folder.fold_expression(unary_postfix.operand)),
            operator: unary_postfix.operator,
        }
    }

    Conditional as conditional => {
        Conditional {
            condition: folder.arena().alloc(folder.fold_expression(conditional.condition)),
            then: conditional.then.map(|then| &*folder.arena().alloc(folder.fold_expression(then))),
            r#else: folder.arena().alloc(folder.fold_expression(conditional.r#else)),
        }
    }

    ArrayElement as array_element => {
        match array_element {
            ArrayElement::KeyValue(key, value) => ArrayElement::KeyValue(
                folder.arena().alloc(folder.fold_expression(key)),
                folder.arena().alloc(folder.fold_expression(value)),
            ),
            ArrayElement::Value(value) => ArrayElement::Value(folder.arena().alloc(folder.fold_expression(value))),
            ArrayElement::Variadic(value) => ArrayElement::Variadic(folder.arena().alloc(folder.fold_expression(value))),
            ArrayElement::Missing => ArrayElement::Missing,
        }
    }

    CompositeStringPart as composite_string_part => {
        match composite_string_part {
            CompositeStringPart::Literal(value) => CompositeStringPart::Literal(value),
            CompositeStringPart::Expression(expression) => {
                CompositeStringPart::Expression(folder.arena().alloc(folder.fold_expression(expression)))
            }
        }
    }

    Instantiation as instantiation => {
        Instantiation {
            class: folder.arena().alloc(folder.fold_expression(instantiation.class)),
            arguments: folder
                .arena()
                .alloc_slice_fill_iter(instantiation.arguments.iter().map(|argument| folder.fold_argument(argument))),
        }
    }

    Call as call => {
        Call {
            callee: folder.fold_callee(&call.callee),
            arguments: folder
                .arena()
                .alloc_slice_fill_iter(call.arguments.iter().map(|argument| folder.fold_argument(argument))),
        }
    }

    PartialApplication as partial_application => {
        PartialApplication {
            callee: folder.fold_callee(&partial_application.callee),
            arguments: folder.arena().alloc_slice_fill_iter(
                partial_application.arguments.iter().map(|argument| folder.fold_partial_argument(argument)),
            ),
        }
    }

    Callee as callee => {
        match callee {
            Callee::Function(expression) => Callee::Function(folder.arena().alloc(folder.fold_expression(expression))),
            Callee::Method(expression, selector) => Callee::Method(
                folder.arena().alloc(folder.fold_expression(expression)),
                folder.fold_member_selector(selector),
            ),
            Callee::NullsafeMethod(expression, selector) => Callee::NullsafeMethod(
                folder.arena().alloc(folder.fold_expression(expression)),
                folder.fold_member_selector(selector),
            ),
            Callee::StaticMethod(expression, selector) => Callee::StaticMethod(
                folder.arena().alloc(folder.fold_expression(expression)),
                folder.fold_member_selector(selector),
            ),
        }
    }

    Access as access => {
        match access {
            Access::Array(target, index) => Access::Array(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.arena().alloc(folder.fold_expression(index)),
            ),
            Access::Property(target, selector) => Access::Property(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.fold_member_selector(selector),
            ),
            Access::NullsafeProperty(target, selector) => Access::NullsafeProperty(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.fold_member_selector(selector),
            ),
            Access::StaticProperty(target, variable) => Access::StaticProperty(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.fold_variable(variable),
            ),
            Access::ClassConstant(target, selector) => Access::ClassConstant(
                folder.arena().alloc(folder.fold_expression(target)),
                folder.fold_constant_selector(selector),
            ),
        }
    }

    MemberSelector as member_selector => {
        match member_selector {
            MemberSelector::Name(name) => MemberSelector::Name(*name),
            MemberSelector::Variable(variable) => MemberSelector::Variable(*variable),
            MemberSelector::Expression(expression) => {
                MemberSelector::Expression(folder.arena().alloc(folder.fold_expression(expression)))
            }
        }
    }

    ConstantSelector as constant_selector => {
        match constant_selector {
            ConstantSelector::Name(name) => ConstantSelector::Name(*name),
            ConstantSelector::Expression(expression) => {
                ConstantSelector::Expression(folder.arena().alloc(folder.fold_expression(expression)))
            }
        }
    }

    Yield as yield_expression => {
        match yield_expression {
            Yield::Nothing => Yield::Nothing,
            Yield::Expression(value) => Yield::Expression(folder.arena().alloc(folder.fold_expression(value))),
            Yield::Pair(key, value) => Yield::Pair(
                folder.arena().alloc(folder.fold_expression(key)),
                folder.arena().alloc(folder.fold_expression(value)),
            ),
            Yield::From(value) => Yield::From(folder.arena().alloc(folder.fold_expression(value))),
        }
    }

    Match as match_expression => {
        Match {
            subject: folder.arena().alloc(folder.fold_expression(match_expression.subject)),
            arms: folder
                .arena()
                .alloc_slice_fill_iter(match_expression.arms.iter().map(|arm| folder.fold_match_arm(arm))),
        }
    }

    MatchArm as match_arm => {
        match match_arm {
            MatchArm::Expression(conditions, body) => MatchArm::Expression(
                folder
                    .arena()
                    .alloc_slice_fill_iter(conditions.iter().map(|condition| folder.fold_expression(condition))),
                folder.arena().alloc(folder.fold_expression(body)),
            ),
            MatchArm::Default(body) => MatchArm::Default(folder.arena().alloc(folder.fold_expression(body))),
        }
    }

    DefinitionExpressionKind as definition_expression_kind => {
        match definition_expression_kind {
            DefinitionExpressionKind::AnonymousClass(node) => {
                DefinitionExpressionKind::AnonymousClass(folder.arena().alloc(folder.fold_anonymous_class(node)))
            }
            DefinitionExpressionKind::ArrowFunction(node) => {
                DefinitionExpressionKind::ArrowFunction(folder.arena().alloc(folder.fold_arrow_function(node)))
            }
            DefinitionExpressionKind::Closure(node) => {
                DefinitionExpressionKind::Closure(folder.arena().alloc(folder.fold_closure(node)))
            }
        }
    }

    AnonymousClass as anonymous_class => {
        AnonymousClass {
            attributes: folder.arena().alloc_slice_fill_iter(
                anonymous_class.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            arguments: folder.arena().alloc_slice_fill_iter(
                anonymous_class.arguments.iter().map(|argument| folder.fold_argument(argument)),
            ),
            extends: anonymous_class.extends,
            extends_annotations: anonymous_class.extends_annotations,
            implements: anonymous_class.implements,
            implements_annotations: anonymous_class.implements_annotations,
            mixin_annotations: anonymous_class.mixin_annotations,
            trait_uses: anonymous_class.trait_uses,
            constants: folder.arena().alloc_slice_fill_iter(
                anonymous_class.constants.iter().map(|constant| folder.fold_class_like_constant(constant)),
            ),
            properties: folder.arena().alloc_slice_fill_iter(
                anonymous_class.properties.iter().map(|property| folder.fold_property(property)),
            ),
            hooked_properties: folder.arena().alloc_slice_fill_iter(
                anonymous_class.hooked_properties.iter().map(|property| folder.fold_hooked_property(property)),
            ),
            methods: folder
                .arena()
                .alloc_slice_fill_iter(anonymous_class.methods.iter().map(|method| folder.fold_method(method))),
        }
    }

    Closure as closure => {
        Closure {
            attributes: folder
                .arena()
                .alloc_slice_fill_iter(closure.attributes.iter().map(|attribute| folder.fold_attribute(attribute))),
            is_static: closure.is_static,
            type_parameter_annotations: closure.type_parameter_annotations,
            parameters: folder
                .arena()
                .alloc_slice_fill_iter(closure.parameters.iter().map(|parameter| folder.fold_parameter(parameter))),
            return_by_reference: closure.return_by_reference,
            return_type: closure.return_type,
            return_type_annotation: closure.return_type_annotation,
            throws_annotations: closure.throws_annotations,
            assert_annotations: closure.assert_annotations,
            assert_if_true_annotations: closure.assert_if_true_annotations,
            assert_if_false_annotations: closure.assert_if_false_annotations,
            use_variables: folder.arena().alloc_slice_fill_iter(closure.use_variables.iter().map(|use_variable| {
                ClosureUseClauseVariable { is_by_reference: use_variable.is_by_reference, variable: use_variable.variable }
            })),
            body: folder.arena().alloc(folder.fold_statement(closure.body)),
        }
    }

    ArrowFunction as arrow_function => {
        ArrowFunction {
            attributes: folder.arena().alloc_slice_fill_iter(
                arrow_function.attributes.iter().map(|attribute| folder.fold_attribute(attribute)),
            ),
            is_static: arrow_function.is_static,
            type_parameter_annotations: arrow_function.type_parameter_annotations,
            parameters: folder.arena().alloc_slice_fill_iter(
                arrow_function.parameters.iter().map(|parameter| folder.fold_parameter(parameter)),
            ),
            return_by_reference: arrow_function.return_by_reference,
            return_type: arrow_function.return_type,
            return_type_annotation: arrow_function.return_type_annotation,
            throws_annotations: arrow_function.throws_annotations,
            assert_annotations: arrow_function.assert_annotations,
            assert_if_true_annotations: arrow_function.assert_if_true_annotations,
            assert_if_false_annotations: arrow_function.assert_if_false_annotations,
            expression: folder.arena().alloc(folder.fold_expression(arrow_function.expression)),
        }
    }

    Variable as variable => {
        match variable {
            Variable::Direct(direct) => Variable::Direct(*direct),
            Variable::Indirect(expression) => {
                Variable::Indirect(folder.arena().alloc(folder.fold_expression(expression)))
            }
            Variable::Nested(nested) => Variable::Nested(folder.arena().alloc(folder.fold_variable(nested))),
        }
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;

    use mago_span::Span;

    use super::Fold;
    use super::StatementFold;
    use crate::ir::IR;
    use crate::ir::expression::ExpressionKind;
    use crate::ir::expression::definition::DefinitionExpressionKind;
    use crate::ir::statement::StatementKind;
    use crate::ir::statement::definition::DefinitionStatementKind;

    struct Widen<'arena> {
        arena: &'arena Bump,
    }

    impl<'arena> Fold<'arena> for Widen<'arena> {
        type FromStatement = String;
        type FromDefinition = String;
        type FromExpression = String;
        type ToStatement = Vec<String>;
        type ToDefinition = ();
        type ToExpression = u32;

        fn arena(&self) -> &'arena Bump {
            self.arena
        }

        fn fold_statement_meta(
            &self,
            _span: Span,
            _kind: &StatementKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>,
        ) -> Self::ToStatement {
            Vec::new()
        }

        fn fold_expression_meta(
            &self,
            _span: Span,
            _kind: &ExpressionKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>,
        ) -> Self::ToExpression {
            0
        }

        fn fold_definition_statement_meta(
            &self,
            _kind: &DefinitionStatementKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>,
        ) -> Self::ToDefinition {
        }

        fn fold_definition_expression_meta(
            &self,
            _kind: &DefinitionExpressionKind<'arena, Self::ToStatement, Self::ToDefinition, Self::ToExpression>,
        ) -> Self::ToDefinition {
        }
    }

    #[test]
    fn folds_between_distinct_non_copy_meta() {
        let arena = Bump::new();
        let widen = Widen { arena: &arena };

        let input: IR<'_, String, String, String> = IR { statements: &[] };
        let output: IR<'_, Vec<String>, (), u32> = widen.fold_ir(&input);

        assert!(output.statements.is_empty());
    }

    struct WidenStatements<'arena> {
        arena: &'arena Bump,
    }

    impl<'arena> StatementFold<'arena, u8, u16> for WidenStatements<'arena> {
        type FromStatement = String;
        type ToStatement = Vec<String>;

        fn arena(&self) -> &'arena Bump {
            self.arena
        }

        fn fold_statement_meta(
            &self,
            _span: Span,
            _kind: &StatementKind<'arena, Self::ToStatement, u8, u16>,
        ) -> Self::ToStatement {
            Vec::new()
        }
    }

    #[test]
    fn statement_fold_changes_only_the_statement_hole() {
        let arena = Bump::new();
        let folder = WidenStatements { arena: &arena };

        let input: IR<'_, String, u8, u16> = IR { statements: &[] };
        let output: IR<'_, Vec<String>, u8, u16> = folder.fold_ir(&input);

        assert!(output.statements.is_empty());
    }
}
