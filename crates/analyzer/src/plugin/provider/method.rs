//! Method return type provider trait.

use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_atom::concat_atom;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MethodTarget {
    pub class: &'static str,
    pub method: &'static str,
}

impl MethodTarget {
    #[inline]
    pub const fn exact(class: &'static str, method: &'static str) -> Self {
        Self { class, method }
    }

    #[inline]
    pub const fn all_methods(class: &'static str) -> Self {
        Self { class, method: "*" }
    }

    #[inline]
    pub const fn any_class(method: &'static str) -> Self {
        Self { class: "*", method }
    }

    pub fn matches(&self, class_name: &str, method_name: &str) -> bool {
        self.matches_class(class_name) && self.matches_method(method_name)
    }

    fn matches_class(&self, class_name: &str) -> bool {
        if self.class == "*" {
            return true;
        }

        if self.class.ends_with('*') {
            let prefix = &self.class[..self.class.len() - 1];
            ascii_lowercase_atom(class_name).as_str().starts_with(ascii_lowercase_atom(prefix).as_str())
        } else {
            class_name.eq_ignore_ascii_case(self.class)
        }
    }

    fn matches_method(&self, method_name: &str) -> bool {
        if self.method == "*" {
            return true;
        }

        if self.method.ends_with('*') {
            let prefix = &self.method[..self.method.len() - 1];
            ascii_lowercase_atom(method_name).as_str().starts_with(ascii_lowercase_atom(prefix).as_str())
        } else {
            method_name.eq_ignore_ascii_case(self.method)
        }
    }

    pub fn is_exact(&self) -> bool {
        !self.class.contains('*') && !self.method.contains('*')
    }

    pub fn index_key(&self) -> Option<Atom> {
        if self.is_exact() {
            Some(concat_atom!(
                ascii_lowercase_atom(self.class).as_str(),
                "::",
                ascii_lowercase_atom(self.method).as_str()
            ))
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
        class_name: &str,
        method_name: &str,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion>;
}
