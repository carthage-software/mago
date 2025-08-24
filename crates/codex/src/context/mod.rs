use mago_atom::Atom;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::reference::ReferenceSource;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopeContext<'ctx> {
    pub(crate) function_like: Option<&'ctx FunctionLikeMetadata>,
    pub(crate) class_like: Option<&'ctx ClassLikeMetadata>,
    pub(crate) is_static: bool,
}

impl Default for ScopeContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ctx> ScopeContext<'ctx> {
    /// Creates a new `ScopeContext` representing a default global, static scope.
    #[inline]
    pub fn new() -> Self {
        Self { function_like: None, class_like: None, is_static: true }
    }

    /// Returns whether the current scope is a global scope.
    #[inline]
    pub const fn is_global(&self) -> bool {
        self.function_like.is_none() && self.class_like.is_none()
    }

    /// Returns whether the current scope is pure.
    #[inline]
    pub const fn is_pure(&self) -> bool {
        if let Some(function_like) = self.function_like
            && function_like.flags.is_pure()
        {
            return true;
        }

        false
    }

    /// Returns the calling class-like context, if available.
    #[inline]
    pub fn get_class_like(&self) -> Option<&'ctx ClassLikeMetadata> {
        self.class_like
    }

    /// Returns the calling class FQCN, if inside a class scope.
    #[inline]
    pub fn get_class_like_name(&self) -> Option<Atom> {
        self.class_like.map(|class| class.original_name)
    }

    /// Returns the calling function-like context, if available.
    #[inline]
    pub fn get_function_like(&self) -> Option<&'ctx FunctionLikeMetadata> {
        self.function_like
    }

    /// Returns the identifier of the calling function/method, if available.
    #[inline]
    pub fn get_function_like_identifier(&self) -> Option<FunctionLikeIdentifier> {
        let function_like = self.function_like?;

        let Some(function_name) = function_like.name else {
            return Some(FunctionLikeIdentifier::Closure(function_like.span.file_id, function_like.span.start));
        };

        Some(if function_like.get_kind().is_method() {
            let Some(class_like) = self.class_like else {
                return Some(FunctionLikeIdentifier::Function(function_name));
            };

            FunctionLikeIdentifier::Method(class_like.name, function_name)
        } else {
            FunctionLikeIdentifier::Function(function_name)
        })
    }

    /// Checks if the calling class scope is marked as `final`.
    #[inline]
    pub const fn is_class_like_final(&self) -> bool {
        match self.class_like {
            Some(class) => class.flags.is_final(),
            None => false,
        }
    }

    /// Checks if the calling scope is static.
    #[inline]
    pub const fn is_static(&self) -> bool {
        self.is_static
    }

    /// Sets the function-like metadata for the current scope.
    #[inline]
    pub fn set_function_like(&mut self, function_like: Option<&'ctx FunctionLikeMetadata>) {
        self.function_like = function_like;
    }

    /// Sets the class-like metadata for the current scope.
    #[inline]
    pub fn set_class_like(&mut self, class_like: Option<&'ctx ClassLikeMetadata>) {
        self.class_like = class_like;
    }

    /// Sets the static flag for the current scope.
    #[inline]
    pub fn set_static(&mut self, is_static: bool) {
        self.is_static = is_static;
    }

    /// Determines the `ReferenceSource` (symbol or member) based on the current function context.
    /// Used to identify the origin of a code reference for dependency tracking.
    #[inline]
    pub fn get_reference_source(&self) -> Option<ReferenceSource> {
        if let Some(calling_functionlike_id) = self.get_function_like_identifier() {
            match calling_functionlike_id {
                FunctionLikeIdentifier::Function(name) => Some(ReferenceSource::Symbol(false, name)),
                FunctionLikeIdentifier::Method(class_name, method_name) => {
                    Some(ReferenceSource::ClassLikeMember(false, class_name, method_name))
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
