use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::argument::ArgumentList;
use crate::ast::attribute::AttributeList;
use crate::ast::class_like::inheritance::Extends;
use crate::ast::class_like::inheritance::Implements;
use crate::ast::class_like::member::ClassLikeMember;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::modifier::Modifier;
use crate::ast::type_hint::Hint;
use crate::sequence::Sequence;

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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Interface {
    pub attribute_lists: Sequence<AttributeList>,
    pub interface: Keyword,
    pub name: LocalIdentifier,
    pub extends: Option<Extends>,
    pub left_brace: Span,
    pub members: Sequence<ClassLikeMember>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Class {
    pub attribute_lists: Sequence<AttributeList>,
    pub modifiers: Sequence<Modifier>,
    pub class: Keyword,
    pub name: LocalIdentifier,
    pub extends: Option<Extends>,
    pub implements: Option<Implements>,
    pub left_brace: Span,
    pub members: Sequence<ClassLikeMember>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AnonymousClass {
    pub new: Keyword,
    pub attribute_lists: Sequence<AttributeList>,
    pub modifiers: Sequence<Modifier>,
    pub class: Keyword,
    pub arguments: Option<ArgumentList>,
    pub extends: Option<Extends>,
    pub implements: Option<Implements>,
    pub left_brace: Span,
    pub members: Sequence<ClassLikeMember>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Trait {
    pub attribute_lists: Sequence<AttributeList>,
    pub r#trait: Keyword,
    pub name: LocalIdentifier,
    pub left_brace: Span,
    pub members: Sequence<ClassLikeMember>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Enum {
    pub attribute_lists: Sequence<AttributeList>,
    pub r#enum: Keyword,
    pub name: LocalIdentifier,
    pub backing_type_hint: Option<EnumBackingTypeHint>,
    pub implements: Option<Implements>,
    pub left_brace: Span,
    pub members: Sequence<ClassLikeMember>,
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct EnumBackingTypeHint {
    pub colon: Span,
    pub hint: Hint,
}

impl HasSpan for Interface {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.right_brace);
        }

        self.interface.span().join(self.right_brace)
    }
}

impl HasSpan for Class {
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

impl HasSpan for AnonymousClass {
    fn span(&self) -> Span {
        self.new.span().join(self.right_brace)
    }
}

impl HasSpan for Trait {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.right_brace);
        }

        self.r#trait.span().join(self.right_brace)
    }
}

impl HasSpan for Enum {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.right_brace);
        }

        self.r#enum.span().join(self.right_brace)
    }
}

impl HasSpan for EnumBackingTypeHint {
    fn span(&self) -> Span {
        Span::between(self.colon, self.hint.span())
    }
}
