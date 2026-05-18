//! Method return type provider trait.

use mago_codex::ttype::union::TUnion;
use mago_word::Word;
use mago_word::ascii_lowercase_word;
use mago_word::concat_word;
use mago_word::starts_with_ignore_case;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodTarget {
    pub class: &'static [u8],
    pub method: &'static [u8],
}

impl MethodTarget {
    #[inline]
    #[must_use]
    pub const fn exact(class: &'static [u8], method: &'static [u8]) -> Self {
        Self { class, method }
    }

    #[inline]
    #[must_use]
    pub const fn all_methods(class: &'static [u8]) -> Self {
        Self { class, method: b"*" }
    }

    #[inline]
    #[must_use]
    pub const fn any_class(method: &'static [u8]) -> Self {
        Self { class: b"*", method }
    }

    #[must_use]
    pub fn matches(&self, class_name: &[u8], method_name: &[u8]) -> bool {
        self.matches_class(class_name) && self.matches_method(method_name)
    }

    fn matches_class(&self, class_name: &[u8]) -> bool {
        if self.class == b"*" {
            return true;
        }

        if self.class.last() == Some(&b'*') {
            starts_with_ignore_case(class_name, &self.class[..self.class.len() - 1])
        } else {
            class_name.eq_ignore_ascii_case(self.class)
        }
    }

    fn matches_method(&self, method_name: &[u8]) -> bool {
        if self.method == b"*" {
            return true;
        }

        if self.method.last() == Some(&b'*') {
            starts_with_ignore_case(method_name, &self.method[..self.method.len() - 1])
        } else {
            method_name.eq_ignore_ascii_case(self.method)
        }
    }

    #[must_use]
    pub fn is_exact(&self) -> bool {
        !self.class.contains(&b'*') && !self.method.contains(&b'*')
    }

    #[must_use]
    pub fn index_key(&self) -> Option<Word> {
        if self.is_exact() {
            Some(concat_word!(ascii_lowercase_word(self.class), b"::", ascii_lowercase_word(self.method)))
        } else {
            None
        }
    }
}

pub trait MethodReturnTypeProvider: Provider {
    fn targets() -> &'static [MethodTarget]
    where
        Self: Sized;

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        class_name: &[u8],
        method_name: &[u8],
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion>;
}
