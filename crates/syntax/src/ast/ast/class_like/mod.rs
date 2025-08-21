use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::argument::ArgumentList;
use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::class_like::inheritance::Extends;
use crate::ast::ast::class_like::inheritance::Implements;
use crate::ast::ast::class_like::member::ClassLikeMember;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::modifier::Modifier;
use crate::ast::ast::type_hint::Hint;
use crate::ast::sequence::Sequence;

pub mod constant;
pub mod enum_case;
pub mod inheritance;
pub mod member;
pub mod method;
pub mod property;
pub mod trait_use;

/// Represents a PHP interface.
///
/// # Example:
///
/// ```php
/// <?php
///
/// interface Foo {}
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Interface<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub interface: Keyword<'arena>,
    pub name: LocalIdentifier<'arena>,
    pub extends: Option<Extends<'arena>>,
    pub left_brace: Span,
    pub members: Sequence<'arena, ClassLikeMember<'arena>>,
    pub right_brace: Span,
}

/// Represents a PHP class.
///
/// # Example:
///
/// ```php
/// <?php
///
/// #[Something(else: 'nothing')]
/// final readonly class Foo extends Bar implements Baz {
///     public function __construct(
///         public string $value
///     ) {}
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Class<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub modifiers: Sequence<'arena, Modifier<'arena>>,
    pub class: Keyword<'arena>,
    pub name: LocalIdentifier<'arena>,
    pub extends: Option<Extends<'arena>>,
    pub implements: Option<Implements<'arena>>,
    pub left_brace: Span,
    pub members: Sequence<'arena, ClassLikeMember<'arena>>,
    pub right_brace: Span,
}

/// Represents a PHP anonymous class.
///
/// # Example:
///
/// ```php
/// <?php
///
/// $instance = new class($foo, $bar) {
///   public function __construct(
///     public string $foo,
///     public int $bar,
///   ) {}
/// };
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct AnonymousClass<'arena> {
    pub new: Keyword<'arena>,
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub modifiers: Sequence<'arena, Modifier<'arena>>,
    pub class: Keyword<'arena>,
    pub argument_list: Option<ArgumentList<'arena>>,
    pub extends: Option<Extends<'arena>>,
    pub implements: Option<Implements<'arena>>,
    pub left_brace: Span,
    pub members: Sequence<'arena, ClassLikeMember<'arena>>,
    pub right_brace: Span,
}

/// Represents a PHP trait.
///
/// # Example:
///
/// ```php
/// <?php
///
/// trait Foo {
///   public function bar(): string {
///     return 'baz';
///   }
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Trait<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub r#trait: Keyword<'arena>,
    pub name: LocalIdentifier<'arena>,
    pub left_brace: Span,
    pub members: Sequence<'arena, ClassLikeMember<'arena>>,
    pub right_brace: Span,
}

/// Represents a PHP enum.
///
/// # Example:
///
/// ```php
/// <?php
///
/// enum Direction {
///   case Up;
///   case Down;
///   case Right;
///   case Left;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Enum<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub r#enum: Keyword<'arena>,
    pub name: LocalIdentifier<'arena>,
    pub backing_type_hint: Option<EnumBackingTypeHint<'arena>>,
    pub implements: Option<Implements<'arena>>,
    pub left_brace: Span,
    pub members: Sequence<'arena, ClassLikeMember<'arena>>,
    pub right_brace: Span,
}

/// Represents a PHP enum backing type hint.
///
/// # Example:
///
/// ```php
/// <?php
///
/// enum LeftOrRight: string {
///   case Left = 'l';
///   case Right = 'r';
/// }
///
/// enum Size: int {
///   case Small = 0;
///   case Medium = 1;
///   case Large = 2;
///   case XLarge = 3;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EnumBackingTypeHint<'arena> {
    pub colon: Span,
    pub hint: Hint<'arena>,
}

impl HasSpan for Interface<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.right_brace);
        }

        self.interface.span().join(self.right_brace)
    }
}

impl HasSpan for Class<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.right_brace);
        }

        if let Some(modifier) = self.modifiers.first() {
            return modifier.span().join(self.right_brace);
        }

        self.class.span().join(self.right_brace)
    }
}

impl HasSpan for AnonymousClass<'_> {
    fn span(&self) -> Span {
        self.new.span().join(self.right_brace)
    }
}

impl HasSpan for Trait<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.right_brace);
        }

        self.r#trait.span().join(self.right_brace)
    }
}

impl HasSpan for Enum<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.right_brace);
        }

        self.r#enum.span().join(self.right_brace)
    }
}

impl HasSpan for EnumBackingTypeHint<'_> {
    fn span(&self) -> Span {
        Span::between(self.colon, self.hint.span())
    }
}
