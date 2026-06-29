use mago_allocator::Arena;
use mago_allocator::CopyInto;
use mago_allocator::vec::Vec;
use mago_hir::ir::delimited::Delimited;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::r#type::annotation::TypeAnnotation;
use mago_oracle::id::SymbolId;
use mago_oracle::linker::lower_type_annotation;
use mago_oracle::ty::Type;

use crate::error::InferenceError;
use crate::error::InferenceResult;
use crate::flow::ControlFlow;
use crate::flow::Flow;
use crate::fold::InferenceFolder;

mod annotation;
mod branch;
mod declare;
mod echo;
mod expression;
mod global;
mod goto;
mod inline;
mod label;
mod namespace;
mod r#return;
mod sequence;
mod r#static;
mod r#try;
mod unset;
mod r#use;

/// The typed statements of a block paired with how the block exits.
type InferredBlock<'arena> = (&'arena [Statement<'arena, SymbolId, Flow, Type<'arena>>], ControlFlow);

impl<'source, 'arena, A, S, E> InferenceFolder<'source, '_, 'arena, A, S, E>
where
    A: Arena,
{
    pub fn infer_statement(
        &mut self,
        statement: &Statement<'source, SymbolId, S, E>,
    ) -> InferenceResult<Statement<'arena, SymbolId, Flow, Type<'arena>>> {
        let first = std::mem::replace(&mut self.is_first_statement, false);
        if !first && requires_first_statement(&statement.kind) {
            self.reachable = false;
        }

        let typed = match statement.kind {
            StatementKind::Expression(expression) => self.infer_expression_statement(statement.span, expression)?,
            StatementKind::Namespace(namespace) => self.infer_namespace(statement.span, namespace)?,
            StatementKind::Sequence(statements) => self.infer_sequence(statement.span, statements)?,
            StatementKind::Return(value) => self.infer_return(statement.span, value)?,
            StatementKind::If(conditional) => self.infer_if(statement.span, conditional)?,
            StatementKind::Echo(expressions) => self.infer_echo(statement.span, expressions)?,
            StatementKind::Global(items) => self.infer_global(statement.span, items)?,
            StatementKind::Static(items) => self.infer_static(statement.span, items)?,
            StatementKind::Unset(Delimited { span: operands_span, items }) => {
                self.infer_unset(statement.span, operands_span, items)?
            }
            StatementKind::VariableBindingAnnotation(annotation) => {
                self.infer_variable_binding_annotation(statement.span, annotation)?
            }
            StatementKind::Inline(content) => self.infer_inline(statement.span, content)?,
            StatementKind::Use(items) => self.infer_use(statement.span, items)?,
            StatementKind::Declare(declare) => self.infer_declare(statement.span, declare)?,
            StatementKind::Goto(label) => self.infer_goto(statement.span, label)?,
            StatementKind::Label(label) => self.infer_label(statement.span, label)?,
            StatementKind::Try(try_statement) => self.infer_try(statement.span, try_statement)?,
            StatementKind::Noop => Statement {
                meta: Flow { reachable: self.reachable, exit: ControlFlow::Fallthrough },
                span: statement.span,
                kind: StatementKind::Noop,
            },
            StatementKind::HaltCompiler => Statement {
                meta: Flow { reachable: self.reachable, exit: ControlFlow::Diverge },
                span: statement.span,
                kind: StatementKind::HaltCompiler,
            },
            StatementKind::Item(_)
            | StatementKind::Foreach(_)
            | StatementKind::For(_)
            | StatementKind::While(_)
            | StatementKind::DoWhile(_)
            | StatementKind::Continue(_)
            | StatementKind::Break(_)
            | StatementKind::Switch(_) => {
                return Err(InferenceError::Unsupported { span: statement.span, construct: "this statement" });
            }
        };

        self.reachable = typed.meta.reachable && matches!(typed.meta.exit, ControlFlow::Fallthrough);

        Ok(typed)
    }

    pub fn infer_block(
        &mut self,
        statements: &'source [Statement<'source, SymbolId, S, E>],
    ) -> InferenceResult<InferredBlock<'arena>> {
        let mut items = Vec::new_in(self.arena);
        let mut exit = ControlFlow::Fallthrough;

        for statement in statements {
            let was_reachable = self.reachable;
            let typed = self.infer_statement(statement)?;
            if was_reachable && !matches!(typed.meta.exit, ControlFlow::Fallthrough) {
                exit = typed.meta.exit;
            }

            items.push(typed);
        }

        Ok((items.leak(), exit))
    }

    /// Lowers an optional `@var`-style type annotation to its type.
    pub(crate) fn lowered_annotation(
        &mut self,
        annotation: Option<&'source TypeAnnotation<'source>>,
    ) -> Option<Type<'arena>> {
        let annotation = annotation?.copy_into(self.arena);

        lower_type_annotation(&mut self.ty, &annotation)
    }
}

/// Whether `kind` is a `declare` PHP requires to be the file's first statement —
/// `declare(strict_types=…)` or `declare(encoding=…)`. Anywhere but first, it is
/// a fatal error, so inference treats it (and what follows) as unreachable.
fn requires_first_statement<I, S, E>(kind: &StatementKind<'_, I, S, E>) -> bool {
    let StatementKind::Declare(declare) = kind else {
        return false;
    };

    declare.items.items.iter().any(|item| matches!(item.name.value, b"strict_types" | b"encoding"))
}
