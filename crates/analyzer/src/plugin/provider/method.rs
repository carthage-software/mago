//! Method return type provider trait.

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
            class_name.to_lowercase().starts_with(&prefix.to_lowercase())
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
            method_name.to_lowercase().starts_with(&prefix.to_lowercase())
        } else {
            method_name.eq_ignore_ascii_case(self.method)
        }
    }

    pub fn is_exact(&self) -> bool {
        !self.class.contains('*') && !self.method.contains('*')
    }

    pub fn index_key(&self) -> Option<String> {
        if self.is_exact() {
            Some(format!("{}::{}", self.class.to_lowercase(), self.method.to_lowercase()))
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
