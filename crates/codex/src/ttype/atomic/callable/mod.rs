use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;
use mago_span::Position;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::callable::parameter::TCallableParameter;
use crate::ttype::get_mixed;
use crate::ttype::union::TUnion;

pub mod parameter;

/// Represents the detailed signature of a PHP `callable` type.
///
/// This includes parameter types and flags, return type, and purity information,
/// often derived from `@param callable(ParamType...): ReturnType` docblock tags
/// or inferred from usage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct TCallableSignature {
    /// `true` if the callable is known to be pure (no side effects), often from `@psalm-pure`.
    pub is_pure: bool,
    /// `true` if this signature specifically represents a closure instance.
    /// May overlap with `closure_position` being `Some`.
    pub is_closure: bool,
    /// Ordered list of parameters expected by the callable signature.
    pub parameters: Vec<TCallableParameter>,
    /// The return type of the callable, if specified. `None` implies `mixed` or unknown.
    pub return_type: Option<Box<TUnion>>, // Keep Box<TUnion> as in original
    /// The source code starting position if this signature originated from a specific closure definition.
    /// `None` if it represents a general callable type not tied to a specific closure literal.
    pub closure_position: Option<Position>,
    /// The source of the callable, if it is an alias or reference to another function-like construct.
    pub source: Option<FunctionLikeIdentifier>,
}

/// Represents a callable entity, which can either be a fully defined signature
/// or an alias pointing to another function, method, or known closure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub enum TCallable {
    /// A concrete callable signature with defined parameters and return type.
    /// Example: Represents `callable(string): int`.
    Signature(TCallableSignature),
    /// An alias or reference to another function-like construct.
    /// Example: Represents `foo(...)`,  `function() { }`.
    Alias(FunctionLikeIdentifier),
}

impl TCallableSignature {
    /// Creates a new `CallableSignature` with the specified purity and closure status.
    #[inline]
    pub fn new(is_pure: bool, is_closure: bool) -> Self {
        Self { is_pure, is_closure, parameters: Vec::new(), return_type: None, closure_position: None, source: None }
    }

    pub fn mixed() -> Self {
        TCallableSignature::new(false, false)
            .with_parameters(vec![TCallableParameter::new(Some(Box::new(get_mixed())), false, true, true)])
            .with_return_type(Some(Box::new(get_mixed())))
    }

    /// Returns a slice of the callable parameters.
    #[inline]
    pub fn get_parameters(&self) -> &[TCallableParameter] {
        &self.parameters
    }

    /// Returns a mutable slice of the callable parameters.
    #[inline]
    pub fn get_parameters_mut(&mut self) -> &mut [TCallableParameter] {
        &mut self.parameters
    }

    /// Returns a reference to the return type (`TUnion`), if specified.
    #[inline]
    pub fn get_return_type(&self) -> Option<&TUnion> {
        self.return_type.as_deref()
    }

    /// Returns a mutable reference to the return type (`TUnion`), if specified.
    #[inline]
    pub fn get_return_type_mut(&mut self) -> Option<&mut TUnion> {
        self.return_type.as_deref_mut()
    }

    /// Returns the closure's starting position, if this signature represents a specific closure literal.
    #[inline]
    pub fn get_closure_position(&self) -> Option<Position> {
        self.closure_position
    }

    /// Checks if the callable is marked as pure.
    #[inline]
    pub const fn is_pure(&self) -> bool {
        self.is_pure
    }

    /// Checks if this signature specifically represents a closure.
    #[inline]
    pub const fn is_closure(&self) -> bool {
        self.is_closure
    }

    /// Returns the source of the callable, if it is an alias or reference to another function-like construct.
    #[inline]
    pub fn get_source(&self) -> Option<FunctionLikeIdentifier> {
        self.source
    }

    /// Clones the signature as a closure, setting `is_closure` to `true`.
    #[inline]
    pub fn clone_as_closure(&self) -> Self {
        Self {
            is_pure: self.is_pure,
            is_closure: true,
            parameters: self.parameters.clone(),
            return_type: self.return_type.clone(),
            closure_position: self.closure_position,
            source: self.source,
        }
    }

    /// Returns a new instance with `is_pure` set to the given value.
    #[inline]
    pub fn with_pure(mut self, is_pure: bool) -> Self {
        self.is_pure = is_pure;
        self
    }

    /// Returns a new instance with the given parameters.
    #[inline]
    pub fn with_parameters(mut self, parameters: Vec<TCallableParameter>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Returns a new instance with the return type set.
    #[inline]
    pub fn with_return_type(mut self, return_type: Option<Box<TUnion>>) -> Self {
        self.return_type = return_type;
        self
    }

    /// Returns a new instance with the closure position set.
    #[inline]
    pub fn with_closure_position(mut self, closure_position: Option<Position>) -> Self {
        self.closure_position = closure_position;
        self
    }

    /// Returns a new instance with the source set.
    #[inline]
    pub fn with_source(mut self, source: Option<FunctionLikeIdentifier>) -> Self {
        self.source = source;
        self
    }
}

impl TCallable {
    /// Checks if this representation is a concrete `Signature`.
    #[inline]
    pub const fn is_signature(&self) -> bool {
        matches!(self, TCallable::Signature(_))
    }

    /// Checks if this representation is an `Alias` to another function-like.
    #[inline]
    pub const fn is_alias(&self) -> bool {
        matches!(self, TCallable::Alias(_))
    }

    /// Returns a reference to the `CallableSignature` if this is the `Signature` variant.
    #[inline]
    pub fn get_signature(&self) -> Option<&TCallableSignature> {
        match self {
            TCallable::Signature(s) => Some(s),
            TCallable::Alias(_) => None,
        }
    }

    /// Returns a reference to the `FunctionLikeIdentifier` if this is the `Alias` variant.
    #[inline]
    pub fn get_alias(&self) -> Option<&FunctionLikeIdentifier> {
        match self {
            TCallable::Signature(_) => None,
            TCallable::Alias(a) => Some(a),
        }
    }
}

impl TType for TCallable {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        let mut children = Vec::new();

        if let TCallable::Signature(signature) = self {
            if let Some(return_type) = &signature.return_type {
                children.push(TypeRef::Union(return_type));
            }

            for parameter in &signature.parameters {
                if let Some(parameter_type) = parameter.get_type_signature() {
                    children.push(TypeRef::Union(parameter_type));
                }
            }
        }

        children
    }

    fn get_id(&self, interner: Option<&ThreadedInterner>) -> String {
        match self {
            TCallable::Signature(signature) => {
                let mut str = String::new();
                str += "(";
                if signature.is_pure() {
                    str += "pure-";
                }

                str += if signature.is_closure() { "closure(" } else { "callable(" };
                for (i, parameter) in signature.get_parameters().iter().enumerate() {
                    if i > 0 {
                        str += ", ";
                    }

                    if parameter.is_variadic() {
                        str += "...";
                    }

                    if let Some(parameter_type) = parameter.get_type_signature() {
                        str += parameter_type.get_id(interner).as_str();
                    } else {
                        str += "mixed";
                    }

                    if parameter.has_default() {
                        str += "=";
                    }
                }

                str += "): ";
                if let Some(return_type) = signature.get_return_type() {
                    str += return_type.get_id(interner).as_str();
                } else {
                    str += "mixed";
                }

                str += ")";

                str
            }
            TCallable::Alias(id) => {
                let mut str = String::from("Closure<");
                if let Some(interner) = interner {
                    str += id.as_string(interner).as_str();
                } else {
                    str += id.to_hash().as_str();
                }

                str += ">(...)";
                str
            }
        }
    }
}
