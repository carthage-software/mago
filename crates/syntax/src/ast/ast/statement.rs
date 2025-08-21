use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::block::Block;
use crate::ast::ast::class_like::Class;
use crate::ast::ast::class_like::Enum;
use crate::ast::ast::class_like::Interface;
use crate::ast::ast::class_like::Trait;
use crate::ast::ast::constant::Constant;
use crate::ast::ast::control_flow::r#if::If;
use crate::ast::ast::control_flow::switch::Switch;
use crate::ast::ast::declare::Declare;
use crate::ast::ast::echo::Echo;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::function_like::function::Function;
use crate::ast::ast::global::Global;
use crate::ast::ast::goto::Goto;
use crate::ast::ast::goto::Label;
use crate::ast::ast::halt_compiler::HaltCompiler;
use crate::ast::ast::inline::Inline;
use crate::ast::ast::r#loop::Break;
use crate::ast::ast::r#loop::Continue;
use crate::ast::ast::r#loop::do_while::DoWhile;
use crate::ast::ast::r#loop::r#for::For;
use crate::ast::ast::r#loop::foreach::Foreach;
use crate::ast::ast::r#loop::r#while::While;
use crate::ast::ast::namespace::Namespace;
use crate::ast::ast::r#return::Return;
use crate::ast::ast::r#static::Static;
use crate::ast::ast::tag::ClosingTag;
use crate::ast::ast::tag::OpeningTag;
use crate::ast::ast::terminator::Terminator;
use crate::ast::ast::r#try::Try;
use crate::ast::ast::unset::Unset;
use crate::ast::ast::r#use::Use;

use super::DeclareBody;
use super::ForBody;
use super::ForeachBody;
use super::IfBody;
use super::NamespaceBody;
use super::WhileBody;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ExpressionStatement<'arena> {
    pub expression: &'arena Expression<'arena>,
    pub terminator: Terminator<'arena>,
}

/// Represents a PHP statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Statement<'arena> {
    OpeningTag(OpeningTag<'arena>),
    ClosingTag(ClosingTag),
    Inline(Inline<'arena>),
    Namespace(Namespace<'arena>),
    Use(Use<'arena>),
    Class(Class<'arena>),
    Interface(Interface<'arena>),
    Trait(Trait<'arena>),
    Enum(Enum<'arena>),
    Block(Block<'arena>),
    Constant(Constant<'arena>),
    Function(Function<'arena>),
    Declare(Declare<'arena>),
    Goto(Goto<'arena>),
    Label(Label<'arena>),
    Try(Try<'arena>),
    Foreach(Foreach<'arena>),
    For(For<'arena>),
    While(While<'arena>),
    DoWhile(DoWhile<'arena>),
    Continue(Continue<'arena>),
    Break(Break<'arena>),
    Switch(Switch<'arena>),
    If(If<'arena>),
    Return(Return<'arena>),
    Expression(ExpressionStatement<'arena>),
    Echo(Echo<'arena>),
    Global(Global<'arena>),
    Static(Static<'arena>),
    HaltCompiler(HaltCompiler<'arena>),
    Unset(Unset<'arena>),
    Noop(Span),
}

impl Statement<'_> {
    #[inline]
    #[must_use]
    pub const fn is_closing_tag(&self) -> bool {
        matches!(self, Statement::ClosingTag(_))
    }

    #[inline]
    #[must_use]
    pub fn terminates_scripting(&self) -> bool {
        match self {
            Statement::ClosingTag(_) => true,
            Statement::Namespace(Namespace { body: NamespaceBody::Implicit(implicit), .. }) => implicit
                .statements
                .last()
                .map_or(implicit.terminator.is_closing_tag(), |statement| statement.terminates_scripting()),
            Statement::Use(r#use) => r#use.terminator.is_closing_tag(),
            Statement::Goto(goto) => goto.terminator.is_closing_tag(),
            Statement::Declare(Declare { body: DeclareBody::Statement(b), .. }) => b.terminates_scripting(),
            Statement::Declare(Declare { body: DeclareBody::ColonDelimited(b), .. }) => b.terminator.is_closing_tag(),
            Statement::For(For { body: ForBody::Statement(b), .. }) => b.terminates_scripting(),
            Statement::For(For { body: ForBody::ColonDelimited(b), .. }) => b.terminator.is_closing_tag(),
            Statement::Foreach(Foreach { body: ForeachBody::Statement(b), .. }) => b.terminates_scripting(),
            Statement::Foreach(Foreach { body: ForeachBody::ColonDelimited(b), .. }) => b.terminator.is_closing_tag(),
            Statement::While(While { body: WhileBody::Statement(b), .. }) => b.terminates_scripting(),
            Statement::While(While { body: WhileBody::ColonDelimited(b), .. }) => b.terminator.is_closing_tag(),
            Statement::DoWhile(do_while) => do_while.terminator.is_closing_tag(),
            Statement::Continue(cont) => cont.terminator.is_closing_tag(),
            Statement::Break(brk) => brk.terminator.is_closing_tag(),
            Statement::If(If { body: IfBody::Statement(stmt), .. }) => match &stmt.else_clause {
                Some(else_clause) => else_clause.statement.terminates_scripting(),
                None => stmt
                    .else_if_clauses
                    .iter()
                    .last()
                    .map_or(stmt.statement.terminates_scripting(), |clause| clause.statement.terminates_scripting()),
            },
            Statement::If(If { body: IfBody::ColonDelimited(body), .. }) => body.terminator.is_closing_tag(),
            Statement::Return(ret) => ret.terminator.is_closing_tag(),
            Statement::Expression(expression_statement) => expression_statement.terminator.is_closing_tag(),
            Statement::Echo(echo) => echo.terminator.is_closing_tag(),
            Statement::Global(global) => global.terminator.is_closing_tag(),
            Statement::Static(r#static) => r#static.terminator.is_closing_tag(),
            Statement::Unset(unset) => unset.terminator.is_closing_tag(),
            Statement::HaltCompiler(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub const fn is_loop(&self) -> bool {
        matches!(self, Statement::For(_) | Statement::Foreach(_) | Statement::While(_) | Statement::DoWhile(_))
    }

    #[inline]
    pub const fn is_control_flow(&self) -> bool {
        matches!(
            self,
            Statement::If(_)
                | Statement::Switch(_)
                | Statement::Try(_)
                | Statement::Continue(_)
                | Statement::Break(_)
                | Statement::Goto(_)
        )
    }

    #[inline]
    pub const fn is_declaration(&self) -> bool {
        matches!(
            self,
            Statement::Declare(_)
                | Statement::Namespace(_)
                | Statement::Class(_)
                | Statement::Interface(_)
                | Statement::Trait(_)
                | Statement::Enum(_)
                | Statement::Constant(_)
                | Statement::Function(_)
        )
    }

    #[inline]
    pub const fn is_noop(&self) -> bool {
        matches!(self, Statement::Noop(_))
    }
}

impl HasSpan for ExpressionStatement<'_> {
    fn span(&self) -> Span {
        self.expression.span().join(self.terminator.span())
    }
}

impl HasSpan for Statement<'_> {
    fn span(&self) -> Span {
        match self {
            Statement::OpeningTag(statement) => statement.span(),
            Statement::ClosingTag(statement) => statement.span(),
            Statement::Inline(statement) => statement.span(),
            Statement::Namespace(statement) => statement.span(),
            Statement::Use(statement) => statement.span(),
            Statement::Class(statement) => statement.span(),
            Statement::Interface(statement) => statement.span(),
            Statement::Trait(statement) => statement.span(),
            Statement::Enum(statement) => statement.span(),
            Statement::Block(statement) => statement.span(),
            Statement::Constant(statement) => statement.span(),
            Statement::Function(statement) => statement.span(),
            Statement::Declare(statement) => statement.span(),
            Statement::Goto(statement) => statement.span(),
            Statement::Label(statement) => statement.span(),
            Statement::Try(statement) => statement.span(),
            Statement::Foreach(statement) => statement.span(),
            Statement::For(statement) => statement.span(),
            Statement::While(statement) => statement.span(),
            Statement::DoWhile(statement) => statement.span(),
            Statement::Continue(statement) => statement.span(),
            Statement::Break(statement) => statement.span(),
            Statement::Switch(statement) => statement.span(),
            Statement::If(statement) => statement.span(),
            Statement::Return(statement) => statement.span(),
            Statement::Expression(statement) => statement.span(),
            Statement::Echo(statement) => statement.span(),
            Statement::Global(statement) => statement.span(),
            Statement::Static(statement) => statement.span(),
            Statement::Unset(statement) => statement.span(),
            Statement::HaltCompiler(statement) => statement.span(),
            Statement::Noop(span) => *span,
        }
    }
}
