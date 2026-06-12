use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;

use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;

pub mod annotation;
pub mod attribute;
pub mod expression;
pub mod inheritance;
pub mod member;
pub mod modifier;
pub mod parameter;
pub mod statement;

pub trait Item<'arena, I, S, E>: Sized + HasSpan {
    fn attributes(&self) -> &'arena [Attribute<'arena, I, S, E>];

    fn annotation(&self) -> Option<&'arena ItemAnnotation<'arena, I, S, E>>;

    fn version_constraint(&self) -> &'arena [PHPVersionRange] {
        &[]
    }
}
