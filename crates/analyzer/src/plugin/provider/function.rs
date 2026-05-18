//! Function return type provider trait.

use mago_codex::ttype::union::TUnion;
use mago_word::starts_with_ignore_case;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionTarget {
    Exact(&'static [u8]),
    ExactMultiple(&'static [&'static [u8]]),
    Prefix(&'static [u8]),
    Namespace(&'static [u8]),
}

impl FunctionTarget {
    #[inline]
    #[must_use]
    pub const fn exact(name: &'static [u8]) -> Self {
        Self::Exact(name)
    }

    #[inline]
    #[must_use]
    pub const fn exact_multiple(names: &'static [&'static [u8]]) -> Self {
        Self::ExactMultiple(names)
    }

    #[inline]
    #[must_use]
    pub const fn prefix(prefix: &'static [u8]) -> Self {
        Self::Prefix(prefix)
    }

    #[inline]
    #[must_use]
    pub const fn namespace(ns: &'static [u8]) -> Self {
        Self::Namespace(ns)
    }

    #[must_use]
    pub fn matches(&self, name: &[u8]) -> bool {
        match self {
            FunctionTarget::Exact(target) => name.eq_ignore_ascii_case(target),
            FunctionTarget::ExactMultiple(targets) => targets.iter().any(|target| name.eq_ignore_ascii_case(target)),
            FunctionTarget::Prefix(prefix) => starts_with_ignore_case(name, prefix),
            FunctionTarget::Namespace(ns) => starts_with_ignore_case(name, ns),
        }
    }

    #[must_use]
    pub fn get_exact_names(&self) -> Option<Vec<&'static [u8]>> {
        match self {
            FunctionTarget::Exact(name) => Some(vec![*name]),
            FunctionTarget::ExactMultiple(names) => Some(names.to_vec()),
            FunctionTarget::Prefix(_) | FunctionTarget::Namespace(_) => None,
        }
    }

    #[must_use]
    pub fn is_prefix(&self) -> bool {
        matches!(self, FunctionTarget::Prefix(_))
    }

    #[must_use]
    pub fn is_namespace(&self) -> bool {
        matches!(self, FunctionTarget::Namespace(_))
    }

    #[must_use]
    pub fn get_prefix(&self) -> Option<&'static [u8]> {
        match self {
            FunctionTarget::Prefix(prefix) => Some(prefix),
            _ => None,
        }
    }

    #[must_use]
    pub fn get_namespace(&self) -> Option<&'static [u8]> {
        match self {
            FunctionTarget::Namespace(ns) => Some(ns),
            _ => None,
        }
    }
}

pub trait FunctionReturnTypeProvider: Provider {
    fn targets() -> FunctionTarget
    where
        Self: Sized;

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion>;
}
