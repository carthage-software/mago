//! Write display text into owned bytes. Compound spellings must not enter the word interner.

use mago_bytes::BytesDisplay;
use mago_word::Word;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::{TGlobalReferenceSelector, TReferenceMemberSelector};
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::union::TUnion;

use super::NameDisplay;

pub(super) fn union(ty: &TUnion, names: Names<'_>) -> Vec<u8> {
    let mut output = Vec::new();
    Renderer { names }.write_union(ty, &mut output);
    output
}

#[derive(Clone, Copy)]
pub(super) enum Names<'names> {
    Canonical,
    Short(&'names NameDisplay<'names>),
    Qualified(&'names CodebaseMetadata),
}

struct Renderer<'names> {
    names: Names<'names>,
}

impl Renderer<'_> {
    fn write_name(&self, name: Word, output: &mut Vec<u8>) {
        match self.names {
            Names::Short(names) => output.extend_from_slice(names.class_name(name).as_bytes()),
            Names::Qualified(codebase) => {
                let name = codebase.get_class_like_by_name(name).map_or(name, |class| class.original_name);
                output.extend_from_slice(name.as_bytes());
            }
            Names::Canonical => output.extend_from_slice(name.as_bytes()),
        }
    }

    fn write_union(&self, ty: &TUnion, output: &mut Vec<u8>) {
        if ty.types.len() <= 1 {
            if let Some(atomic) = ty.types.first() {
                self.write_wrapped_atomic(atomic, self.wrap_generic(atomic), output);
            }
            return;
        }

        let mut parts: Vec<_> = ty
            .types
            .iter()
            .map(|atomic| {
                let mut part = Vec::new();
                self.write_wrapped_atomic(
                    atomic,
                    self.wrap_generic(atomic) || atomic.has_intersection_types(),
                    &mut part,
                );
                part
            })
            .collect();
        parts.sort_unstable();
        parts.dedup();
        for (index, part) in parts.iter().enumerate() {
            if index > 0 {
                output.push(b'|');
            }
            output.extend_from_slice(part);
        }
    }

    fn wrap_generic(&self, ty: &TAtomic) -> bool {
        matches!(ty, TAtomic::GenericParameter(parameter)
            if self.show_template_bound(&parameter.constraint))
    }

    fn show_template_bound(&self, constraint: &TUnion) -> bool {
        matches!(self.names, Names::Canonical)
            || !matches!(constraint.types.as_ref(), [TAtomic::Mixed(mixed)] if mixed.is_vanilla())
    }

    fn write_wrapped_atomic(&self, ty: &TAtomic, wrapped: bool, output: &mut Vec<u8>) {
        if wrapped {
            output.push(b'(');
        }
        self.write_atomic(ty, output);
        if wrapped {
            output.push(b')');
        }
    }

    fn write_intersections(&self, types: Option<&[TAtomic]>, wrap_nested: bool, output: &mut Vec<u8>) {
        for ty in types.into_iter().flatten() {
            output.push(b'&');
            self.write_wrapped_atomic(ty, wrap_nested && ty.has_intersection_types(), output);
        }
    }

    fn write_atomic(&self, ty: &TAtomic, output: &mut Vec<u8>) {
        match ty {
            TAtomic::Object(TObject::Named(object)) => {
                if object.is_this {
                    output.extend_from_slice(b"$this(");
                }
                self.write_name(object.name, output);
                if let Some(parameters) = object.get_type_parameters() {
                    output.push(b'<');
                    self.write_types(parameters.iter(), output);
                    output.push(b'>');
                }
                self.write_intersections(object.get_intersection_types(), true, output);
                if object.is_this {
                    output.push(b')');
                } else if object.is_static {
                    output.extend_from_slice(b"&static");
                }
            }
            TAtomic::Object(TObject::Enum(object)) => {
                let shortened = matches!(self.names, Names::Short(_));
                if !shortened {
                    output.extend_from_slice(b"enum(");
                }

                self.write_name(object.name, output);
                if let Some(case) = object.case {
                    output.extend_from_slice(b"::");
                    if shortened {
                        output.extend_from_slice(BytesDisplay(case.as_bytes()).to_string().as_bytes());
                    } else {
                        output.extend_from_slice(case.as_bytes());
                    }
                }

                if !shortened {
                    output.push(b')');
                }
            }
            TAtomic::Object(TObject::WithProperties(object)) => {
                output.extend_from_slice(b"object{");
                for (index, (name, (optional, ty))) in object.known_properties.iter().enumerate() {
                    if index > 0 {
                        output.extend_from_slice(b", ");
                    }

                    output.extend_from_slice(name.as_bytes());
                    if *optional {
                        output.push(b'?');
                    }

                    output.extend_from_slice(b": ");
                    self.write_union(ty, output);
                }

                if !object.sealed {
                    if !object.known_properties.is_empty() {
                        output.extend_from_slice(b", ");
                    }

                    output.extend_from_slice(b"...");
                }

                output.push(b'}');
            }
            TAtomic::Object(TObject::HasMethod(object)) => {
                output.extend_from_slice(b"has-method<'");
                output.extend_from_slice(object.method.as_bytes());
                output.extend_from_slice(b"'>");
                self.write_intersections(object.get_intersection_types(), true, output);
            }
            TAtomic::Object(TObject::HasProperty(object)) => {
                output.extend_from_slice(b"has-property<'");
                output.extend_from_slice(object.property.as_bytes());
                output.extend_from_slice(b"'>");
                self.write_intersections(object.get_intersection_types(), true, output);
            }
            TAtomic::Reference(reference) => self.write_reference(reference, output),
            TAtomic::Alias(alias) => {
                output.push(b'!');
                self.write_name(alias.get_class_name(), output);
                output.extend_from_slice(b"::");
                output.extend_from_slice(alias.get_alias_name().as_bytes());
            }
            TAtomic::Scalar(TScalar::ClassLikeString(string)) => self.write_class_string(string, output),
            TAtomic::Array(array) => self.write_array(array, output),
            TAtomic::Callable(callable) => self.write_callable(callable, output),
            TAtomic::GenericParameter(parameter) => {
                let bounded = self.show_template_bound(&parameter.constraint);
                let wrapped = bounded && parameter.intersection_types.is_some();
                if wrapped {
                    output.push(b'(');
                }

                match self.names {
                    Names::Canonical => {
                        output.push(b'\'');
                        output.extend_from_slice(parameter.parameter_name.as_bytes());
                        output.push(b'.');
                        write_parent(&parameter.defining_entity, output);
                        output.extend_from_slice(b" extends ");
                        self.write_union(&parameter.constraint, output);
                    }
                    Names::Short(names) => {
                        names.write_template_name(parameter, output);
                        if bounded {
                            output.extend_from_slice(b" of ");
                            self.write_union(&parameter.constraint, output);
                        }
                    }
                    Names::Qualified(codebase) => {
                        NameDisplay::fully_qualified(codebase).write_template_name(parameter, output);
                        if bounded {
                            output.extend_from_slice(b" of ");
                            self.write_union(&parameter.constraint, output);
                        }
                    }
                }
                if wrapped {
                    output.push(b')');
                }

                self.write_intersections(parameter.get_intersection_types(), true, output);
            }
            TAtomic::Iterable(iterable) => {
                if iterable.intersection_types.is_some() {
                    output.push(b'(');
                }

                output.extend_from_slice(b"iterable<");
                self.write_types([iterable.get_key_type(), iterable.get_value_type()].into_iter(), output);
                output.push(b'>');
                if iterable.intersection_types.is_some() {
                    output.push(b')');
                }

                self.write_intersections(iterable.get_intersection_types(), false, output);
            }
            TAtomic::Conditional(conditional) => {
                output.push(b'(');
                self.write_union(&conditional.subject, output);
                output.extend_from_slice(if conditional.negated { b" is not " } else { b" is " });
                self.write_union(&conditional.target, output);
                output.extend_from_slice(b" ? ");
                self.write_union(&conditional.then, output);
                output.extend_from_slice(b" : ");
                self.write_union(&conditional.otherwise, output);
                output.push(b')');
            }
            TAtomic::Derived(derived) => self.write_derived(derived, output),
            TAtomic::Scalar(scalar) => write_scalar(scalar, output),
            TAtomic::Object(TObject::Any)
            | TAtomic::Mixed(_)
            | TAtomic::Resource(_)
            | TAtomic::Variable(_)
            | TAtomic::Never
            | TAtomic::Null
            | TAtomic::Void
            | TAtomic::Placeholder => output.extend_from_slice(ty.get_id().as_bytes()),
        }
    }

    fn write_types<'ty>(&self, types: impl Iterator<Item = &'ty TUnion>, output: &mut Vec<u8>) {
        for (index, ty) in types.enumerate() {
            if index > 0 {
                output.extend_from_slice(b", ");
            }

            self.write_union(ty, output);
        }
    }

    fn write_callable(&self, callable: &TCallable, output: &mut Vec<u8>) {
        match callable {
            TCallable::Signature(signature) => {
                output.push(b'(');
                if signature.is_pure() {
                    output.extend_from_slice(b"pure-");
                }

                output.extend_from_slice(if signature.is_closure() { b"closure(" } else { b"callable(" });
                for (index, parameter) in signature.get_parameters().iter().enumerate() {
                    if index > 0 {
                        output.extend_from_slice(b", ");
                    }

                    if parameter.is_variadic() {
                        output.extend_from_slice(b"...");
                    }

                    self.write_optional_type(parameter.get_type_signature(), output);
                    if parameter.has_default() {
                        output.push(b'=');
                    }
                }

                output.extend_from_slice(b"): ");
                self.write_optional_type(signature.get_return_type(), output);
                output.push(b')');
            }
            TCallable::Alias(key) => {
                output.extend_from_slice(b"Closure<");
                match key {
                    FunctionLikeIdentifier::Function(name) | FunctionLikeIdentifier::Closure(name) => {
                        output.extend_from_slice(name.as_bytes())
                    }
                    FunctionLikeIdentifier::Method(class, method) => {
                        output.extend_from_slice(class.as_bytes());
                        output.extend_from_slice(b"::");
                        output.extend_from_slice(method.as_bytes());
                    }
                }

                output.extend_from_slice(b">(...)");
            }
        }
    }

    fn write_optional_type(&self, ty: Option<&TUnion>, output: &mut Vec<u8>) {
        if let Some(ty) = ty {
            self.write_union(ty, output);
        } else {
            output.extend_from_slice(b"mixed");
        }
    }

    fn write_class_string(&self, string: &TClassLikeString, output: &mut Vec<u8>) {
        match string {
            TClassLikeString::Any { kind } => output.extend_from_slice(kind.as_bytes()),
            TClassLikeString::Literal { value } => {
                output.extend_from_slice(b"class-string('");
                output.extend_from_slice(value.as_bytes());
                output.extend_from_slice(b"')");
            }
            TClassLikeString::Generic { kind, parameter_name, defining_entity, constraint } => {
                output.extend_from_slice(kind.as_bytes());
                output.extend_from_slice(b"<'");
                output.extend_from_slice(parameter_name.as_bytes());
                output.push(b'.');
                write_parent(defining_entity, output);
                output.extend_from_slice(b" extends ");
                self.write_atomic(constraint, output);
                output.push(b'>');
            }
            TClassLikeString::OfType { kind, constraint } => {
                output.extend_from_slice(kind.as_bytes());
                output.push(b'<');
                self.write_atomic(constraint, output);
                output.push(b'>');
            }
        }
    }

    fn write_reference(&self, reference: &TReference, output: &mut Vec<u8>) {
        match reference {
            TReference::Symbol { name, .. } => {
                output.extend_from_slice(b"unknown-ref(");
                self.write_name(*name, output);
            }
            TReference::Member { class_like_name, member_selector } => {
                output.extend_from_slice(b"unknown-ref(");
                self.write_name(*class_like_name, output);
                output.extend_from_slice(b"::");
                match member_selector {
                    TReferenceMemberSelector::Wildcard => output.push(b'*'),
                    TReferenceMemberSelector::Identifier(name) => output.extend_from_slice(name.as_bytes()),
                    TReferenceMemberSelector::StartsWith(name) => {
                        output.extend_from_slice(name.as_bytes());
                        output.push(b'*');
                    }
                    TReferenceMemberSelector::EndsWith(name) => {
                        output.push(b'*');
                        output.extend_from_slice(name.as_bytes());
                    }
                }
            }
            TReference::Global { selector } => {
                output.extend_from_slice(b"unknown-global-ref(");
                match selector {
                    TGlobalReferenceSelector::Identifier(name) => output.extend_from_slice(name.as_bytes()),
                    TGlobalReferenceSelector::Fallback { name, fallback } => {
                        output.extend_from_slice(name.as_bytes());
                        output.extend_from_slice(b" ?? ");
                        output.extend_from_slice(fallback.as_bytes());
                    }
                    TGlobalReferenceSelector::StartsWith(name) => {
                        output.extend_from_slice(name.as_bytes());
                        output.push(b'*');
                    }
                    TGlobalReferenceSelector::EndsWith(name) => {
                        output.push(b'*');
                        output.extend_from_slice(name.as_bytes());
                    }
                }
            }
        }

        output.push(b')');
    }
}

fn write_parent(parent: &GenericParent, output: &mut Vec<u8>) {
    match parent {
        GenericParent::ClassLike(name) => output.extend_from_slice(name.as_bytes()),
        GenericParent::FunctionLike((class, name)) => {
            if !class.is_empty() {
                output.extend_from_slice(class.as_bytes());
                output.extend_from_slice(b"::");
            }

            output.extend_from_slice(name.as_bytes());
            output.extend_from_slice(b"()");
        }
    }
}

mod arrays;
mod derived;
mod scalars;

use scalars::write_scalar;
