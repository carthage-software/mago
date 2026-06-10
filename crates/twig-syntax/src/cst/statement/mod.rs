use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

pub use crate::cst::statement::apply::*;
pub use crate::cst::statement::autoescape::*;
pub use crate::cst::statement::block::*;
pub use crate::cst::statement::cache::*;
pub use crate::cst::statement::deprecated::*;
pub use crate::cst::statement::do_::*;
pub use crate::cst::statement::embed::*;
pub use crate::cst::statement::extends::*;
pub use crate::cst::statement::flush::*;
pub use crate::cst::statement::r#for::*;
pub use crate::cst::statement::from::*;
pub use crate::cst::statement::guard::*;
pub use crate::cst::statement::r#if::*;
pub use crate::cst::statement::import::*;
pub use crate::cst::statement::include::*;
pub use crate::cst::statement::r#macro::*;
pub use crate::cst::statement::print::*;
pub use crate::cst::statement::sandbox::*;
pub use crate::cst::statement::set::*;
pub use crate::cst::statement::text::*;
pub use crate::cst::statement::types::*;
pub use crate::cst::statement::unknown::*;
pub use crate::cst::statement::r#use::*;
pub use crate::cst::statement::verbatim::*;
pub use crate::cst::statement::with::*;

pub mod apply;
pub mod autoescape;
pub mod block;
pub mod cache;
pub mod deprecated;
pub mod do_;
pub mod embed;
pub mod extends;
pub mod flush;
pub mod r#for;
pub mod from;
pub mod guard;
pub mod r#if;
pub mod import;
pub mod include;
pub mod r#macro;
pub mod print;
pub mod sandbox;
pub mod set;
pub mod text;
pub mod types;
pub mod unknown;
pub mod r#use;
pub mod verbatim;
pub mod with;

/// A Twig statement.
///
/// Statements are the significant constructs that live at template body
/// level: raw template text, `{{ expr }}` prints, `{% verbatim %}` blocks,
/// and every `{% tag %}` form. Template-level comments (`{# ... #}`) and
/// inline expression comments (`# ...`) are **not** statements - they live
/// on [`Template::trivia`](crate::cst::Template) alongside whitespace.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum Statement<'arena> {
    /// Raw template text between tags.
    Text(Text<'arena>),
    /// `{{ expr }}` - a print/output statement.
    Print(Print<'arena>),
    /// `{% verbatim %}...{% endverbatim %}` (or `{% raw %}...{% endraw %}`).
    Verbatim(Verbatim<'arena>),
    /// `{% if %} ... {% endif %}` with optional elseif/else branches.
    If(If<'arena>),
    /// `{% for %} ... {% endfor %}` (with optional else).
    For(For<'arena>),
    /// `{% set %}` - assignment or capture block.
    Set(Set<'arena>),
    /// `{% block %}` - named block definition.
    Block(Block<'arena>),
    /// `{% extends %}`.
    Extends(Extends<'arena>),
    /// `{% use %}`.
    Use(Use<'arena>),
    /// `{% include %}`.
    Include(Include<'arena>),
    /// `{% embed %} ... {% endembed %}`.
    Embed(Embed<'arena>),
    /// `{% import %}`.
    Import(Import<'arena>),
    /// `{% from %}`.
    From(From<'arena>),
    /// `{% macro %} ... {% endmacro %}`.
    Macro(Macro<'arena>),
    /// `{% with %} ... {% endwith %}`.
    With(With<'arena>),
    /// `{% apply %} ... {% endapply %}`.
    Apply(Apply<'arena>),
    /// `{% autoescape %} ... {% endautoescape %}`.
    Autoescape(Autoescape<'arena>),
    /// `{% sandbox %} ... {% endsandbox %}`.
    Sandbox(Sandbox<'arena>),
    /// `{% deprecated %}`.
    Deprecated(Deprecated<'arena>),
    /// `{% do %}`.
    Do(Do<'arena>),
    /// `{% flush %}`.
    Flush(Flush<'arena>),
    /// `{% guard %}`.
    Guard(Guard<'arena>),
    /// `{% cache %} ... {% endcache %}`.
    Cache(Cache<'arena>),
    /// `{% types %}`.
    Types(Types<'arena>),
    /// A structurally well-formed tag whose name is not recognised. Reserved
    /// for future extensibility; the parser currently errors on unknown tags.
    Unknown(Unknown<'arena>),
}

impl HasSpan for Statement<'_> {
    fn span(&self) -> Span {
        match self {
            Statement::Text(n) => n.span(),
            Statement::Print(n) => n.span(),
            Statement::Verbatim(n) => n.span(),
            Statement::If(n) => n.span(),
            Statement::For(n) => n.span(),
            Statement::Set(n) => n.span(),
            Statement::Block(n) => n.span(),
            Statement::Extends(n) => n.span(),
            Statement::Use(n) => n.span(),
            Statement::Include(n) => n.span(),
            Statement::Embed(n) => n.span(),
            Statement::Import(n) => n.span(),
            Statement::From(n) => n.span(),
            Statement::Macro(n) => n.span(),
            Statement::With(n) => n.span(),
            Statement::Apply(n) => n.span(),
            Statement::Autoescape(n) => n.span(),
            Statement::Sandbox(n) => n.span(),
            Statement::Deprecated(n) => n.span(),
            Statement::Do(n) => n.span(),
            Statement::Flush(n) => n.span(),
            Statement::Guard(n) => n.span(),
            Statement::Cache(n) => n.span(),
            Statement::Types(n) => n.span(),
            Statement::Unknown(n) => n.span(),
        }
    }
}
