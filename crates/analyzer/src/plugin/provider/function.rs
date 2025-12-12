//! Function return type provider trait.

use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionTarget {
    Exact(&'static str),
    ExactMultiple(&'static [&'static str]),
    Prefix(&'static str),
    Namespace(&'static str),
}

impl FunctionTarget {
    #[inline]
    pub const fn exact(name: &'static str) -> Self {
        Self::Exact(name)
    }

    #[inline]
    pub const fn exact_multiple(names: &'static [&'static str]) -> Self {
        Self::ExactMultiple(names)
    }

    #[inline]
    pub const fn prefix(prefix: &'static str) -> Self {
        Self::Prefix(prefix)
    }

    #[inline]
    pub const fn namespace(ns: &'static str) -> Self {
        Self::Namespace(ns)
    }

    pub fn matches(&self, name: &str) -> bool {
        match self {
            FunctionTarget::Exact(target) => name.eq_ignore_ascii_case(target),
            FunctionTarget::ExactMultiple(targets) => targets.iter().any(|target| name.eq_ignore_ascii_case(target)),
            FunctionTarget::Prefix(prefix) => name.to_lowercase().starts_with(&prefix.to_lowercase()),
            FunctionTarget::Namespace(ns) => name.to_lowercase().starts_with(&ns.to_lowercase()),
        }
    }

    pub fn get_exact_names(&self) -> Option<Vec<&'static str>> {
        match self {
            FunctionTarget::Exact(name) => Some(vec![*name]),
            FunctionTarget::ExactMultiple(names) => Some(names.to_vec()),
            FunctionTarget::Prefix(_) | FunctionTarget::Namespace(_) => None,
        }
    }

    pub fn is_prefix(&self) -> bool {
        matches!(self, FunctionTarget::Prefix(_))
    }

    pub fn is_namespace(&self) -> bool {
        matches!(self, FunctionTarget::Namespace(_))
    }

    pub fn get_prefix(&self) -> Option<&'static str> {
        match self {
            FunctionTarget::Prefix(prefix) => Some(prefix),
            _ => None,
        }
    }

    pub fn get_namespace(&self) -> Option<&'static str> {
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
