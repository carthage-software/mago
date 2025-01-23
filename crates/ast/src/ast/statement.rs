use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::block::Block;
use crate::ast::class_like::Class;
use crate::ast::class_like::Enum;
use crate::ast::class_like::Interface;
use crate::ast::class_like::Trait;
use crate::ast::constant::Constant;
use crate::ast::control_flow::r#if::If;
use crate::ast::control_flow::switch::Switch;
use crate::ast::declare::Declare;
use crate::ast::echo::Echo;
use crate::ast::expression::Expression;
use crate::ast::function_like::function::Function;
use crate::ast::global::Global;
use crate::ast::goto::Goto;
use crate::ast::goto::Label;
use crate::ast::halt_compiler::HaltCompiler;
use crate::ast::inline::Inline;
use crate::ast::namespace::Namespace;
use crate::ast::r#loop::do_while::DoWhile;
use crate::ast::r#loop::foreach::Foreach;
use crate::ast::r#loop::r#for::For;
use crate::ast::r#loop::r#while::While;
use crate::ast::r#loop::Break;
use crate::ast::r#loop::Continue;
use crate::ast::r#return::Return;
use crate::ast::r#static::Static;
use crate::ast::r#try::Try;
use crate::ast::r#use::Use;
use crate::ast::tag::ClosingTag;
use crate::ast::tag::OpeningTag;
use crate::ast::terminator::Terminator;
use crate::ast::unset::Unset;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ExpressionStatement {
    pub expression: Box<Expression>,
    pub terminator: Terminator,
}

/// Represents a PHP statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
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
