use bumpalo::boxed::Box;
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

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ExpressionStatement<'a> {
    pub expression: Box<'a, Expression<'a>>,
    pub terminator: Terminator,
}

/// Represents a PHP statement.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Statement<'a> {
    OpeningTag(OpeningTag),
    ClosingTag(ClosingTag),
    Inline(Inline),
    Namespace(Namespace<'a>),
    Use(Use<'a>),
    Class(Class<'a>),
    Interface(Interface<'a>),
    Trait(Trait<'a>),
    Enum(Enum<'a>),
    Block(Block<'a>),
    Constant(Constant<'a>),
    Function(Function<'a>),
    Declare(Declare<'a>),
    Goto(Goto),
    Label(Label),
    Try(Try<'a>),
    Foreach(Foreach<'a>),
    For(For<'a>),
    While(While<'a>),
    DoWhile(DoWhile<'a>),
    Continue(Continue<'a>),
    Break(Break<'a>),
    Switch(Switch<'a>),
    If(If<'a>),
    Return(Return<'a>),
    Expression(ExpressionStatement<'a>),
    Echo(Echo<'a>),
    Global(Global<'a>),
    Static(Static<'a>),
    HaltCompiler(HaltCompiler),
    Unset(Unset<'a>),
    Noop(Span),
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
