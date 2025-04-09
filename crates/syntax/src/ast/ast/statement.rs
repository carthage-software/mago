use serde::Deserialize;
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ExpressionStatement {
    pub expression: Box<Expression>,
    pub terminator: Terminator,
}

/// Represents a PHP statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Statement {
    OpeningTag(OpeningTag),
    ClosingTag(ClosingTag),
    Inline(Inline),
    Namespace(Namespace),
    Use(Use),
    Class(Class),
    Interface(Interface),
    Trait(Trait),
    Enum(Enum),
    Block(Block),
    Constant(Constant),
    Function(Function),
    Declare(Declare),
    Goto(Goto),
    Label(Label),
    Try(Try),
    Foreach(Foreach),
    For(For),
    While(While),
    DoWhile(DoWhile),
    Continue(Continue),
    Break(Break),
    Switch(Switch),
    If(If),
    Return(Return),
    Expression(ExpressionStatement),
    Echo(Echo),
    Global(Global),
    Static(Static),
    HaltCompiler(HaltCompiler),
    Unset(Unset),
    Noop(Span),
}

impl Statement {
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
}

impl HasSpan for ExpressionStatement {
    fn span(&self) -> Span {
        self.expression.span().join(self.terminator.span())
    }
}

impl HasSpan for Statement {
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
