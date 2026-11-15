//! A call's remaining parameters and their original declarations.

use std::borrow::Cow;

use mago_analyzer::artifacts::editor::{BoundParameter, BoundSignature};
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::ttype::display::{NameDisplay, canonical_bytes};

use super::targets::SignatureTarget;

#[derive(Clone, Copy)]
pub(crate) struct CallSignature<'a> {
    pub function: &'a FunctionLikeMetadata,
    bound: Option<&'a BoundSignature>,
}

#[derive(Clone, Copy)]
pub(crate) struct CallParameter<'a> {
    pub declaration: &'a FunctionLikeParameterMetadata,
    bound: Option<&'a BoundParameter>,
}

impl SignatureTarget {
    pub(crate) fn signature<'a>(&'a self, codebase: &'a CodebaseMetadata) -> Option<CallSignature<'a>> {
        let function = codebase.get_function_like(&self.key)?;
        let signature = CallSignature { function, bound: self.bound.as_deref() };
        // A missing or unmatched name cannot identify the source parameter.
        (signature.bound.is_none() || signature.parameters().count() == signature.len()).then_some(signature)
    }
}

impl<'a> CallSignature<'a> {
    pub(crate) fn len(self) -> usize {
        self.bound.map_or(self.function.parameters.len(), |bound| bound.parameters.len())
    }

    pub(crate) fn parameters(self) -> impl Iterator<Item = CallParameter<'a>> {
        (0..self.len()).filter_map(move |index| {
            if let Some(bound) = self.bound {
                let parameter = &bound.parameters[index];
                let name = parameter.name?;
                let declaration = self.function.parameters.iter().find(|parameter| parameter.name.0 == name)?;
                Some(CallParameter { declaration, bound: Some(parameter) })
            } else {
                Some(CallParameter { declaration: &self.function.parameters[index], bound: None })
            }
        })
    }

    pub(crate) fn parameter(self, name: &[u8]) -> Option<CallParameter<'a>> {
        self.parameters().find(|parameter| parameter.declaration.name.0.as_bytes().strip_prefix(b"$") == Some(name))
    }

    pub(crate) fn return_type(self, names: Option<&NameDisplay<'_>>) -> Option<Cow<'a, [u8]>> {
        self.bound.map_or_else(
            || {
                self.function.return_type_metadata.as_ref().map(|ty| {
                    Cow::Owned(names.map_or_else(
                        || canonical_bytes(&ty.type_union),
                        |names| names.union(&ty.type_union).into_bytes(),
                    ))
                })
            },
            |bound| bound.return_type.as_deref().map(|text| Cow::Borrowed(text.as_bytes())),
        )
    }
}

impl<'a> CallParameter<'a> {
    pub(crate) fn type_name(self, names: Option<&NameDisplay<'_>>) -> Option<Cow<'a, [u8]>> {
        self.bound.map_or_else(
            || {
                self.declaration.type_metadata.as_ref().map(|ty| {
                    Cow::Owned(names.map_or_else(
                        || canonical_bytes(&ty.type_union),
                        |names| names.union(&ty.type_union).into_bytes(),
                    ))
                })
            },
            |bound| bound.type_name.as_deref().map(|text| Cow::Borrowed(text.as_bytes())),
        )
    }

    pub(crate) fn flags(self) -> MetadataFlags {
        let mut flags = self.declaration.flags;
        if let Some(bound) = self.bound {
            flags.set(MetadataFlags::BY_REFERENCE, bound.by_reference);
            flags.set(MetadataFlags::VARIADIC, bound.variadic);
            flags.set(MetadataFlags::HAS_DEFAULT, bound.has_default);
        }
        flags
    }
}

#[cfg(test)]
mod tests {
    use mago_codex::metadata::function_like::FunctionLikeKind;
    use mago_codex::metadata::ttype::TypeMetadata;
    use mago_codex::misc::VariableIdentifier;
    use mago_codex::ttype::atomic::{
        TAtomic,
        object::{TObject, named::TNamedObject},
    };
    use mago_codex::ttype::union::TUnion;
    use mago_span::Span;
    use mago_word::word;

    use super::*;

    #[test]
    fn full_type_spelling_is_owned_and_bound_text_is_borrowed() {
        let span = Span::dummy(0, 0);
        let ty = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            word("Domain\\Box"),
            Some(vec![TUnion::from_atomic(TAtomic::Object(TObject::new_enum_case(
                word("Domain\\State"),
                word("Ready"),
            )))]),
        ))));
        let mut function = FunctionLikeMetadata::new(
            FunctionLikeKind::Function,
            word("read"),
            word("read"),
            span,
            MetadataFlags::empty(),
        );
        function.return_type_metadata = Some(TypeMetadata::new(ty.clone(), span));
        let mut parameter =
            FunctionLikeParameterMetadata::new(VariableIdentifier(word("$value")), span, span, MetadataFlags::empty());
        parameter.type_metadata = Some(TypeMetadata::new(ty, span));
        function.parameters.push(parameter);
        let signature = CallSignature { function: &function, bound: None };
        let Some(Cow::Owned(return_type)) = signature.return_type(None) else {
            panic!("full return type must own its spelling")
        };
        let Some(Cow::Owned(parameter_type)) =
            signature.parameters().next().and_then(|parameter| parameter.type_name(None))
        else {
            panic!("full parameter type must own its spelling")
        };
        assert_eq!(return_type, b"Domain\\Box<enum(Domain\\State::Ready)>");
        assert_eq!(parameter_type, return_type);

        let bound = BoundParameter {
            name: Some(word("$value")),
            type_name: Some("Box<State::Ready>".into()),
            by_reference: false,
            variadic: false,
            has_default: false,
        };
        let parameter = CallParameter { declaration: &function.parameters[0], bound: Some(&bound) };
        assert!(matches!(parameter.type_name(None), Some(Cow::Borrowed(b"Box<State::Ready>"))));
        drop(function);
        assert_eq!(return_type, parameter_type, "owned text must outlive the declaration");
    }
}
