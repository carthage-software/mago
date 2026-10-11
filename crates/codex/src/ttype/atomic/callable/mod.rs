use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::LazyLock;

use mago_word::Word;
use mago_word::concat_word;
use mago_word::word;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::misc::VariableIdentifier;
use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::callable::parameter::TCallableParameter;
use crate::ttype::get_mixed;
use crate::ttype::union::TUnion;

pub mod parameter;

// Keep a thin pointer without allocating a separate empty vector for each signature.
#[allow(clippy::rc_buffer)]
static EMPTY_CONSTRAINTS: LazyLock<Arc<Vec<TCallableConstraint>>> = LazyLock::new(|| Arc::new(Vec::new()));

/// Represents the detailed signature of a PHP `callable` type.
///
/// This includes parameter types and flags, return type, and purity information,
/// often derived from `@param callable(ParamType...): ReturnType` docblock tags
/// or inferred from usage.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::box_collection, reason = "Keep the collection header off the stack to shrink every TAtomic.")]
pub struct TCallableSignature {
    /// `true` if the callable is known to be pure (no side effects), often from `@psalm-pure`.
    pub is_pure: bool,
    /// `true` if this signature specifically represents a closure instance.
    pub is_closure: bool,
    /// The source has bound arguments; its full parameter list must not replace this signature.
    #[cfg_attr(feature = "serde", serde(default))]
    pub is_partial: bool,
    /// Ordered list of parameters expected by the callable signature.
    pub parameters: Box<Vec<TCallableParameter>>,
    /// The return type of the callable, if specified. `None` implies `mixed` or unknown.
    pub return_type: Option<Arc<TUnion>>,
    /// The invoked declaration. Use `get_source()` when recovering its full signature.
    pub source: Option<FunctionLikeIdentifier>,
    /// Captured arguments whose validity depends on parameters that will be
    /// supplied when this callable is invoked.
    pub constraints: Arc<Vec<TCallableConstraint>>,
}

impl PartialEq for TCallableSignature {
    fn eq(&self, other: &Self) -> bool {
        self.is_pure == other.is_pure
            && self.is_closure == other.is_closure
            && self.is_partial == other.is_partial
            && self.parameters == other.parameters
            && self.return_type == other.return_type
            && self.source == other.source
            && self.constraints == other.constraints
            && self.partial_parameter_names().eq(other.partial_parameter_names())
    }
}

impl Eq for TCallableSignature {}

impl Hash for TCallableSignature {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (
            self.is_pure,
            self.is_closure,
            self.is_partial,
            &self.parameters,
            &self.return_type,
            self.source,
            &self.constraints,
        )
            .hash(state);
        for name in self.partial_parameter_names() {
            name.hash(state);
        }
    }
}

impl PartialOrd for TCallableSignature {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TCallableSignature {
    fn cmp(&self, other: &Self) -> Ordering {
        self.is_pure
            .cmp(&other.is_pure)
            .then_with(|| self.is_closure.cmp(&other.is_closure))
            .then_with(|| self.is_partial.cmp(&other.is_partial))
            .then_with(|| self.parameters.cmp(&other.parameters))
            .then_with(|| self.return_type.cmp(&other.return_type))
            .then_with(|| self.source.cmp(&other.source))
            .then_with(|| self.constraints.cmp(&other.constraints))
            .then_with(|| self.partial_parameter_names().cmp(other.partial_parameter_names()))
    }
}

/// A captured argument check deferred by a partial application.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TCallableConstraint {
    /// Parameters of this callable referenced by `parameter_type`.
    pub parameter_names: Vec<VariableIdentifier>,
    /// The type of the argument captured by the partial application.
    pub input_type: Arc<TUnion>,
    /// The original parameter type, with unbound callable parameters retained.
    pub parameter_type: Arc<TUnion>,
}

impl TCallableConstraint {
    #[inline]
    #[must_use]
    pub fn new(parameter_names: Vec<VariableIdentifier>, input_type: Arc<TUnion>, parameter_type: Arc<TUnion>) -> Self {
        Self { parameter_names, input_type, parameter_type }
    }
}

/// Represents a callable entity, which can either be a fully defined signature
/// or an alias pointing to another function, method, or known closure.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TCallable {
    /// A concrete callable signature with defined parameters and return type.
    /// Example: Represents `callable(string): int`.
    Signature(TCallableSignature),
    /// An alias or reference to another function-like construct.
    /// Example: Represents `foo(...)`,  `function() { }`.
    Alias(FunctionLikeIdentifier),
}

impl TCallableSignature {
    fn partial_parameter_names(&self) -> impl Iterator<Item = Option<&VariableIdentifier>> {
        let parameters = if self.is_partial { self.parameters.as_slice() } else { &[] };
        parameters.iter().map(TCallableParameter::get_name)
    }

    /// Creates a new `CallableSignature` with the specified purity and closure status.
    #[inline]
    #[must_use]
    pub fn new(is_pure: bool, is_closure: bool) -> Self {
        Self {
            is_pure,
            is_closure,
            is_partial: false,
            parameters: Box::default(),
            return_type: None,
            source: None,
            constraints: Arc::clone(&EMPTY_CONSTRAINTS),
        }
    }

    #[must_use]
    pub fn mixed(is_closure: bool) -> Self {
        TCallableSignature::new(false, is_closure)
            .with_parameters(vec![TCallableParameter::new(Some(Arc::new(get_mixed())), false, true, true)])
            .with_return_type(Some(Arc::new(get_mixed())))
    }

    /// Returns a slice of the callable parameters.
    #[inline]
    #[must_use]
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
    #[must_use]
    pub fn get_return_type(&self) -> Option<&TUnion> {
        self.return_type.as_deref()
    }

    /// Returns a mutable reference to the return type (`TUnion`), if specified.
    #[inline]
    pub fn get_return_type_mut(&mut self) -> Option<&mut TUnion> {
        self.return_type.as_mut().map(Arc::make_mut)
    }

    /// Checks if the callable is marked as pure.
    #[inline]
    #[must_use]
    pub const fn is_pure(&self) -> bool {
        self.is_pure
    }

    /// Checks if this signature specifically represents a closure.
    #[inline]
    #[must_use]
    pub const fn is_closure(&self) -> bool {
        self.is_closure
    }

    /// Returns the declaration only when its full signature still applies.
    #[inline]
    #[must_use]
    pub fn get_source(&self) -> Option<FunctionLikeIdentifier> {
        if self.is_partial { None } else { self.source }
    }

    /// Returns the invoked declaration, including the source of a partial application.
    #[inline]
    #[must_use]
    pub fn invocation_target(&self) -> Option<FunctionLikeIdentifier> {
        self.source
    }

    /// Returns a new instance with the given parameters.
    #[inline]
    #[must_use]
    pub fn with_parameters(mut self, parameters: Vec<TCallableParameter>) -> Self {
        self.parameters = Box::new(parameters);
        self
    }

    /// Returns a new instance with the return type set.
    #[inline]
    #[must_use]
    pub fn with_return_type(mut self, return_type: Option<Arc<TUnion>>) -> Self {
        self.return_type = return_type;
        self
    }

    /// Returns a new instance with the source set.
    #[inline]
    #[must_use]
    pub fn with_source(mut self, source: Option<FunctionLikeIdentifier>) -> Self {
        self.source = source;
        self.is_partial = false;
        self
    }

    /// Retains a partial application's declaration without restoring its bound parameters.
    #[inline]
    #[must_use]
    pub fn with_partial_source(mut self, source: Option<FunctionLikeIdentifier>) -> Self {
        self.source = source;
        self.is_partial = source.is_some();
        self
    }

    /// Returns a new signature with deferred captured-argument constraints.
    #[inline]
    #[must_use]
    pub fn with_constraints(mut self, constraints: Vec<TCallableConstraint>) -> Self {
        self.constraints = if constraints.is_empty() { Arc::clone(&EMPTY_CONSTRAINTS) } else { Arc::new(constraints) };
        self
    }
}

impl TCallable {
    #[inline]
    #[must_use]
    pub const fn is_closure(&self) -> bool {
        match self {
            TCallable::Signature(signature) => signature.is_closure(),
            TCallable::Alias(identifier) => identifier.is_closure(),
        }
    }

    /// Returns a reference to the `CallableSignature` if this is the `Signature` variant.
    #[inline]
    #[must_use]
    pub fn get_signature(&self) -> Option<&TCallableSignature> {
        match self {
            TCallable::Signature(s) => Some(s),
            TCallable::Alias(_) => None,
        }
    }

    /// Returns a reference to the `FunctionLikeIdentifier` if this is the `Alias` variant.
    #[inline]
    #[must_use]
    pub fn get_alias(&self) -> Option<&FunctionLikeIdentifier> {
        match self {
            TCallable::Signature(_) => None,
            TCallable::Alias(a) => Some(a),
        }
    }
}

impl TType for TCallable {
    fn append_child_nodes<'types>(&'types self, children: &mut Vec<TypeRef<'types>>) {
        if let TCallable::Signature(signature) = self {
            if let Some(return_type) = &signature.return_type {
                children.push(TypeRef::Union(return_type));
            }

            for parameter in signature.parameters.iter() {
                for parameter_type in parameter
                    .get_type_signature()
                    .into_iter()
                    .chain(parameter.get_out_type())
                    .chain(parameter.get_closure_this_type())
                {
                    children.push(TypeRef::Union(parameter_type));
                }
            }

            for constraint in signature.constraints.iter() {
                children.push(TypeRef::Union(&constraint.input_type));
                children.push(TypeRef::Union(&constraint.parameter_type));
            }
        }
    }

    fn needs_population(&self) -> bool {
        match self {
            TCallable::Signature(signature) => {
                signature.return_type.as_ref().is_some_and(|ty| ty.needs_population())
                    || signature.parameters.iter().any(|param| {
                        param
                            .get_type_signature()
                            .into_iter()
                            .chain(param.get_out_type())
                            .chain(param.get_closure_this_type())
                            .any(super::super::TType::needs_population)
                    })
                    || signature.constraints.iter().any(|constraint| {
                        constraint.input_type.needs_population() || constraint.parameter_type.needs_population()
                    })
            }
            TCallable::Alias(_) => false,
        }
    }

    fn is_expandable(&self) -> bool {
        match self {
            TCallable::Alias(_) => true,
            TCallable::Signature(signature) => {
                signature.return_type.as_ref().is_some_and(|ty| ty.is_expandable())
                    || signature.parameters.iter().any(|param| {
                        param
                            .get_type_signature()
                            .into_iter()
                            .chain(param.get_out_type())
                            .chain(param.get_closure_this_type())
                            .any(super::super::TType::is_expandable)
                    })
                    || signature.constraints.iter().any(|constraint| {
                        constraint.input_type.is_expandable() || constraint.parameter_type.is_expandable()
                    })
            }
        }
    }

    fn is_complex(&self) -> bool {
        match self {
            TCallable::Alias(_) => false,
            TCallable::Signature(signature) => {
                signature.parameters.len() > 2
                    || signature.return_type.as_ref().is_some_and(|ty| ty.is_complex())
                    || signature
                        .parameters
                        .iter()
                        .any(|param| param.get_type_signature().is_some_and(super::super::TType::is_complex))
            }
        }
    }

    fn get_id(&self) -> Word {
        match self {
            TCallable::Signature(signature) => {
                let mut buf: Vec<u8> = Vec::new();
                buf.extend_from_slice(b"(");
                if signature.is_pure() {
                    buf.extend_from_slice(b"pure-");
                }

                buf.extend_from_slice(if signature.is_closure() { b"closure(" } else { b"callable(" });
                for (i, parameter) in signature.get_parameters().iter().enumerate() {
                    if i > 0 {
                        buf.extend_from_slice(b", ");
                    }

                    if parameter.is_variadic() {
                        buf.extend_from_slice(b"...");
                    }

                    if let Some(parameter_type) = parameter.get_type_signature() {
                        buf.extend_from_slice(parameter_type.get_id().as_bytes());
                    } else {
                        buf.extend_from_slice(b"mixed");
                    }

                    if parameter.has_default() {
                        buf.extend_from_slice(b"=");
                    }
                }

                buf.extend_from_slice(b"): ");
                if let Some(return_type) = signature.get_return_type() {
                    buf.extend_from_slice(return_type.get_id().as_bytes());
                } else {
                    buf.extend_from_slice(b"mixed");
                }

                buf.extend_from_slice(b")");

                word(&buf)
            }
            TCallable::Alias(id) => match id {
                FunctionLikeIdentifier::Function(fn_name) => {
                    concat_word!(b"Closure<", fn_name.as_bytes(), b">(...)")
                }
                FunctionLikeIdentifier::Method(fqcn, method_name) => {
                    concat_word!(b"Closure<", fqcn.as_bytes(), b"::", method_name.as_bytes(), b">(...)")
                }
                FunctionLikeIdentifier::Closure(name) => {
                    concat_word!(b"Closure<", name.as_bytes(), b">(...)")
                }
            },
        }
    }

    fn get_pretty_id_with_indent(&self, indent: usize) -> Word {
        match self {
            TCallable::Signature(signature) => {
                let parameters = signature.get_parameters();

                // Use multiline format if there are more than 2 parameters
                if parameters.len() > 2 {
                    let mut buf: Vec<u8> = Vec::new();
                    buf.extend_from_slice(b"(");
                    if signature.is_pure() {
                        buf.extend_from_slice(b"pure-");
                    }
                    buf.extend_from_slice(if signature.is_closure() { b"closure(\n" } else { b"callable(\n" });

                    let param_indent = indent + 2;

                    for (i, parameter) in parameters.iter().enumerate() {
                        if i > 0 {
                            buf.extend_from_slice(b",\n");
                        }

                        buf.resize(buf.len() + param_indent, b' ');
                        if parameter.is_variadic() {
                            buf.extend_from_slice(b"...");
                        }

                        if let Some(parameter_type) = parameter.get_type_signature() {
                            buf.extend_from_slice(parameter_type.get_pretty_id_with_indent(param_indent).as_bytes());
                        } else {
                            buf.extend_from_slice(b"mixed");
                        }

                        if parameter.has_default() {
                            buf.extend_from_slice(b"=");
                        }
                    }

                    buf.extend_from_slice(b"\n");
                    buf.resize(buf.len() + indent, b' ');
                    buf.extend_from_slice(b"): ");
                    if let Some(return_type) = signature.get_return_type() {
                        buf.extend_from_slice(return_type.get_pretty_id_with_indent(indent).as_bytes());
                    } else {
                        buf.extend_from_slice(b"mixed");
                    }
                    buf.extend_from_slice(b")");

                    word(&buf)
                } else {
                    self.get_id()
                }
            }
            TCallable::Alias(_) => self.get_id(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use super::*;

    #[test]
    fn partial_parameter_names_participate_in_identity_only_for_partials() {
        let key = FunctionLikeIdentifier::Function(word("read"));
        let signature = |name| {
            TCallableSignature::new(false, true)
                .with_source(Some(key))
                .with_parameters(vec![TCallableParameter::default().with_name(Some(VariableIdentifier(word(name))))])
        };
        let left = signature("$value");
        let right = signature("$other");
        assert_eq!(left, right);
        assert_eq!(left.cmp(&right), Ordering::Equal);
        assert_eq!(HashSet::from([left.clone(), right.clone()]).len(), 1);
        let left = left.with_partial_source(Some(key));
        let right = right.with_partial_source(Some(key));
        assert_ne!(left, right);
        assert_ne!(left.cmp(&right), Ordering::Equal);
        assert_eq!(left.cmp(&right), right.cmp(&left).reverse());
        assert_eq!(HashSet::from([left.clone(), right.clone(), left.clone()]).len(), 2);
        assert_eq!(BTreeSet::from([left.clone(), right, left]).len(), 2);
    }

    #[test]
    fn callable_out_types_participate_in_identity_and_traversal() {
        use crate::metadata::CodebaseMetadata;
        use crate::reference::SymbolReferences;
        use crate::symbol::SymbolKind;
        use crate::ttype::atomic::TAtomic;
        use crate::ttype::atomic::object::{TObject, named::TNamedObject};
        use crate::ttype::atomic::populate_atomic_type;
        use crate::ttype::atomic::reference::TReference;

        let named = |name| TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(word(name)))));
        let parameter = TCallableParameter::new(None, true, false, false);
        let left = parameter.clone().with_out_type(Some(Arc::new(named("FirstBase"))));
        let right = parameter.clone().with_out_type(Some(Arc::new(named("SecondBase"))));
        assert_ne!(left, right);
        assert_ne!(left.cmp(&right), Ordering::Equal);
        assert_eq!(HashSet::from([left.clone(), right.clone(), left.clone()]).len(), 2);
        assert_eq!(BTreeSet::from([left.clone(), right, left]).len(), 2);

        let reference = TUnion::from_atomic(TAtomic::Reference(TReference::Symbol {
            name: word("Output"),
            parameters: None,
            variances: None,
            intersection_types: None,
        }));
        let parameter =
            parameter.with_out_type(Some(Arc::new(reference))).with_closure_this_type(Some(Arc::new(named("parent"))));
        let mut atomic = TAtomic::Callable(TCallable::Signature(
            TCallableSignature::new(false, true).with_parameters(vec![parameter]),
        ));
        assert!(atomic.needs_population());
        assert!(atomic.is_expandable());
        assert!(atomic.any_child_node(|node| matches!(node, TypeRef::Atomic(TAtomic::Reference(_)))));

        let mut codebase = CodebaseMetadata::new();
        codebase.symbols.add_symbol_name(word("output"), SymbolKind::Class);
        populate_atomic_type(&mut atomic, &codebase.symbols, None, &mut SymbolReferences::new(), false);
        let TAtomic::Callable(TCallable::Signature(signature)) = atomic else { panic!("callable signature") };
        assert_eq!(signature.parameters[0].get_out_type(), Some(&named("Output")));
    }

    #[test]
    fn callable_effects_follow_inferred_templates() {
        use crate::metadata::CodebaseMetadata;
        use crate::misc::GenericParent;
        use crate::ttype::atomic::TAtomic;
        use crate::ttype::atomic::generic::TGenericParameter;
        use crate::ttype::get_int;
        use crate::ttype::get_mixed;
        use crate::ttype::template::{TemplateResult, inferred_type_replacer};

        let parent = GenericParent::ClassLike(word("Factory"));
        let template = Arc::new(TUnion::from_atomic(TAtomic::GenericParameter(TGenericParameter::new(
            word("T"),
            Arc::new(get_mixed()),
            parent,
        ))));
        let parameter = TCallableParameter::new(None, true, false, false)
            .with_out_type(Some(Arc::clone(&template)))
            .with_closure_this_type(Some(template));
        let original = TUnion::from_atomic(TAtomic::Callable(TCallable::Signature(
            TCallableSignature::new(false, true).with_parameters(vec![parameter]),
        )));
        assert!(original.has_template_types());
        let mut templates = TemplateResult::default();
        templates.add_lower_bound(word("T"), parent, get_int());
        let replaced = inferred_type_replacer::replace(&original, &templates, &CodebaseMetadata::new());
        let TAtomic::Callable(TCallable::Signature(signature)) = &replaced.types[0] else {
            panic!("callable signature")
        };
        assert_eq!(signature.parameters[0].get_out_type(), Some(&get_int()));
        assert_eq!(signature.parameters[0].get_closure_this_type(), Some(&get_int()));
        assert!(original.has_template_types());
        assert!(!replaced.has_template_types());
    }
}
