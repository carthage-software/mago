use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::block::Block;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::function_like::parameter::FunctionLikeParameterList;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::modifier::Modifier;
use crate::ast::ast::terminator::Terminator;
use crate::ast::ast::type_hint::Hint;
use crate::ast::ast::variable::DirectVariable;

use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Property<'arena> {
    Plain(PlainProperty<'arena>),
    Hooked(HookedProperty<'arena>),
}

/// Represents a class-like property declaration in PHP.
///
/// # Examples
///
/// ```php
/// <?php
///
/// class Example {
///    public $foo;
///    protected $bar = 42;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PlainProperty<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub modifiers: Sequence<'arena, Modifier<'arena>>,
    pub var: Option<Keyword<'arena>>,
    pub hint: Option<Hint<'arena>>,
    pub items: TokenSeparatedSequence<'arena, PropertyItem<'arena>>,
    pub terminator: Terminator<'arena>,
}

/// Represents a class-like property declaration in PHP with hooks.
///
/// # Examples
///
/// ```php
/// <?php
///
/// class Example {
///   private $_foo;
///
///   public $foo {
///     get() {
///        return $this->_foo;
///     }
///     set($value) {
///       $this->_foo = $value;
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct HookedProperty<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub modifiers: Sequence<'arena, Modifier<'arena>>,
    pub var: Option<Keyword<'arena>>,
    pub hint: Option<Hint<'arena>>,
    pub item: PropertyItem<'arena>,
    pub hook_list: PropertyHookList<'arena>,
}

/// Represents a property item in a class-like property declaration in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum PropertyItem<'arena> {
    Abstract(PropertyAbstractItem<'arena>),
    Concrete(PropertyConcreteItem<'arena>),
}

/// Represents an abstract property item in a class-like property declaration in PHP.
///
/// # Examples
///
/// ```php
/// <?php
///
/// class Example {
///    public $foo;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyAbstractItem<'arena> {
    pub variable: DirectVariable<'arena>,
}

/// Represents a concrete property item in a class-like property declaration in PHP.
///
/// # Examples
///
/// ```php
/// <?php
///
/// class Example {
///   public $foo = 42;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyConcreteItem<'arena> {
    pub variable: DirectVariable<'arena>,
    pub equals: Span,
    pub value: Expression<'arena>,
}

/// Represents a list of property hooks in a class-like property declaration in PHP.
///
/// # Examples
///
/// ```php
/// <?php
///
/// class Example {
///   public $foo {
///     get() {
///       return $this->bar;
///     }
///     set($value) {
///       $this->bar = $value;
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyHookList<'arena> {
    pub left_brace: Span,
    pub hooks: Sequence<'arena, PropertyHook<'arena>>,
    pub right_brace: Span,
}

/// Represents a property hook in a class-like property declaration in PHP.
///
/// # Examples
///
/// ```php
/// <?php
///
/// class Example {
///   public $foo {
///     get() {
///       return $this->bar;
///     }
///   }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyHook<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub modifiers: Sequence<'arena, Modifier<'arena>>,
    pub ampersand: Option<Span>,
    pub name: LocalIdentifier<'arena>,
    pub parameters: Option<FunctionLikeParameterList<'arena>>,
    pub body: PropertyHookBody<'arena>,
}

/// Represents the body of a property hook in a class-like property declaration in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum PropertyHookBody<'arena> {
    Abstract(PropertyHookAbstractBody),
    Concrete(PropertyHookConcreteBody<'arena>),
}

/// Represents an abstract body of a property hook in a class-like property declaration in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyHookAbstractBody {
    pub semicolon: Span,
}

/// Represents a concrete body of a property hook in a class-like property declaration in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum PropertyHookConcreteBody<'arena> {
    Block(Block<'arena>),
    Expression(PropertyHookConcreteExpressionBody<'arena>),
}

/// Represents an expression body of a property hook in a class-like property declaration in PHP.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyHookConcreteExpressionBody<'arena> {
    pub arrow: Span,
    pub expression: Expression<'arena>,
    pub semicolon: Span,
}

impl<'arena> Property<'arena> {
    pub fn modifiers(&self) -> &Sequence<'arena, Modifier<'arena>> {
        match &self {
            Property::Hooked(h) => &h.modifiers,
            Property::Plain(p) => &p.modifiers,
        }
    }

    pub fn var(&self) -> Option<&Keyword<'arena>> {
        match &self {
            Property::Hooked(h) => h.var.as_ref(),
            Property::Plain(p) => p.var.as_ref(),
        }
    }

    pub fn first_variable(&self) -> &DirectVariable<'arena> {
        self.variables()
            .first()
            .expect("expected property to have at least 1 item. this is a bug in mago. please report it.")
    }

    pub fn variables(&self) -> Vec<&DirectVariable<'arena>> {
        match &self {
            Property::Plain(inner) => inner.items.iter().map(|item| item.variable()).collect(),
            Property::Hooked(inner) => vec![inner.item.variable()],
        }
    }

    pub fn hint(&self) -> Option<&Hint<'arena>> {
        match &self {
            Property::Hooked(h) => h.hint.as_ref(),
            Property::Plain(p) => p.hint.as_ref(),
        }
    }
}

impl<'arena> PropertyItem<'arena> {
    pub fn variable(&self) -> &DirectVariable<'arena> {
        match &self {
            PropertyItem::Abstract(item) => &item.variable,
            PropertyItem::Concrete(item) => &item.variable,
        }
    }
}

impl HasSpan for Property<'_> {
    fn span(&self) -> Span {
        match &self {
            Property::Plain(inner) => inner.span(),
            Property::Hooked(inner) => inner.span(),
        }
    }
}

impl HasSpan for PlainProperty<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.terminator.span());
        }

        match (self.modifiers.first(), &self.var) {
            (Some(modifier), Some(var)) => {
                if var.span().start < modifier.span().start {
                    return var.span().join(self.terminator.span());
                }

                return modifier.span().join(self.terminator.span());
            }
            (Some(modifier), _) => return modifier.span().join(self.terminator.span()),
            (_, Some(var)) => return var.span().join(self.terminator.span()),
            _ => {}
        }

        if let Some(type_hint) = &self.hint {
            return type_hint.span().join(self.terminator.span());
        }

        if let Some(item) = self.items.first() {
            return item.span().join(self.terminator.span());
        }

        self.terminator.span()
    }
}

impl HasSpan for HookedProperty<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return Span::between(attribute_list.span(), self.hook_list.span());
        }

        match (self.modifiers.first(), &self.var) {
            (Some(modifiers), Some(var)) => {
                if var.span().start < modifiers.span().start {
                    return Span::between(var.span(), self.hook_list.span());
                }

                return Span::between(modifiers.span(), self.hook_list.span());
            }
            (Some(modifiers), _) => return Span::between(modifiers.span(), self.hook_list.span()),
            (_, Some(var)) => return Span::between(var.span(), self.hook_list.span()),
            _ => {}
        }

        if let Some(type_hint) = &self.hint {
            return Span::between(type_hint.span(), self.hook_list.span());
        }

        Span::between(self.item.span(), self.hook_list.span())
    }
}

impl HasSpan for PropertyItem<'_> {
    fn span(&self) -> Span {
        match self {
            PropertyItem::Abstract(item) => item.span(),
            PropertyItem::Concrete(item) => item.span(),
        }
    }
}

impl HasSpan for PropertyAbstractItem<'_> {
    fn span(&self) -> Span {
        self.variable.span()
    }
}

impl HasSpan for PropertyConcreteItem<'_> {
    fn span(&self) -> Span {
        Span::between(self.variable.span(), self.value.span())
    }
}

impl HasSpan for PropertyHookList<'_> {
    fn span(&self) -> Span {
        Span::between(self.left_brace, self.right_brace)
    }
}

impl HasSpan for PropertyHook<'_> {
    fn span(&self) -> Span {
        if let Some(attributes) = self.attribute_lists.first() {
            return Span::between(attributes.span(), self.body.span());
        }

        if let Some(modifier) = self.modifiers.first() {
            return Span::between(modifier.span(), self.body.span());
        }

        if let Some(ampersand) = self.ampersand {
            return Span::between(ampersand, self.body.span());
        }

        Span::between(self.name.span(), self.body.span())
    }
}

impl HasSpan for PropertyHookBody<'_> {
    fn span(&self) -> Span {
        match self {
            PropertyHookBody::Abstract(body) => body.span(),
            PropertyHookBody::Concrete(body) => body.span(),
        }
    }
}

impl HasSpan for PropertyHookAbstractBody {
    fn span(&self) -> Span {
        self.semicolon
    }
}

impl HasSpan for PropertyHookConcreteBody<'_> {
    fn span(&self) -> Span {
        match self {
            PropertyHookConcreteBody::Block(body) => body.span(),
            PropertyHookConcreteBody::Expression(body) => body.span(),
        }
    }
}

impl HasSpan for PropertyHookConcreteExpressionBody<'_> {
    fn span(&self) -> Span {
        Span::between(self.arrow, self.semicolon)
    }
}
