pub mod alias;
pub mod effect;
pub mod generics;
pub mod inheritance;
pub mod member;
pub mod parameter;

use mago_allocator::Arena;
use mago_allocator::vec::Vec;

use mago_flags::U32Flags;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::Element;
use mago_phpdoc_syntax::cst::Tag;
use mago_phpdoc_syntax::cst::tag::TagValue;
use mago_phpdoc_syntax::cst::tag::TagVendor;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::error::annotation::AnnotationError;
use crate::ir::identifier::Identifier;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::annotation::ItemAnnotationTag;
use crate::ir::item::annotation::effect::AssertAnnotation;
use crate::ir::item::annotation::generics::TypeParameterAnnotation;
use crate::ir::item::annotation::generics::Variance;
use crate::ir::item::annotation::member::PropertyAnnotationKind;
use crate::ir::item::annotation::parameter::ParameterAnnotation;
use crate::ir::item::annotation::parameter::ParameterOutAnnotation;
use crate::ir::variable::annotation::VariableAnnotation;
use crate::lower::Lowering;

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_item_annotation(
        &mut self,
        document: Option<&Document<'scratch>>,
        owner: Option<Identifier<'arena>>,
    ) -> Option<&'arena ItemAnnotation<'arena, (), (), ()>> {
        let type_parameters = self.register_item_type_parameters(document, owner);

        self.build_item_annotation(document, owner, type_parameters, None).0
    }

    pub(crate) fn register_item_type_parameters(
        &mut self,
        document: Option<&Document<'scratch>>,
        owner: Option<Identifier<'arena>>,
    ) -> &'arena [TypeParameterAnnotation<'arena>] {
        let mut type_parameters = Vec::new_in(self.arena);
        for element in document.into_iter().flat_map(|document| document.elements.iter()) {
            let Element::Tag(tag) = element else { continue };
            let tag = *tag;
            match &tag.value {
                TagValue::TypeAlias(alias) => {
                    if let Some(owner) = owner {
                        let name = self.phpdoc_name(&alias.alias);
                        self.type_resolution.add_alias(name.value, owner, name);
                        if self.settings.program_wide_type_aliases {
                            self.type_resolution.add_program_alias(name.value, owner, name);
                        }
                    }
                }
                TagValue::TypeAliasImport(import) => {
                    if owner.is_some() {
                        let from = self.resolve_phpdoc_class(&import.imported_from);
                        let name = self.phpdoc_name(&import.imported_alias);
                        let local = import
                            .imported_as
                            .as_ref()
                            .map_or(name.value, |imported_as| self.interner.intern(imported_as.local.value));
                        self.type_resolution.add_alias(local, from, name);
                        if self.settings.re_export_type_aliases {
                            self.type_resolution.add_program_alias(local, from, name);
                        }
                    }
                }
                TagValue::Template(template) => {
                    let annotation = self.lower_type_parameter_annotation(template, Variance::from(template.variance));
                    self.type_resolution.add_template(annotation.name, annotation.bound, annotation.default);
                    type_parameters.push(annotation);
                }
                _ => {}
            }
        }

        type_parameters.leak()
    }

    pub(crate) fn build_item_annotation(
        &mut self,
        document: Option<&Document<'scratch>>,
        owner: Option<Identifier<'arena>>,
        type_parameters: &'arena [TypeParameterAnnotation<'arena>],
        inferred_assertions: Option<(&'arena [AssertAnnotation<'arena>], &'arena [AssertAnnotation<'arena>])>,
    ) -> (Option<&'arena ItemAnnotation<'arena, (), (), ()>>, bool) {
        let arena = self.arena;

        let mut tags = U32Flags::new();
        if document.is_some_and(|document| document.has_inherit_doc()) {
            tags.set(ItemAnnotationTag::InheritDoc);
        }

        let mut type_aliases = Vec::new_in(arena);
        let mut imported_type_aliases = Vec::new_in(arena);
        let mut extends = Vec::new_in(arena);
        let mut require_extends = Vec::new_in(arena);
        let mut implements = Vec::new_in(arena);
        let mut require_implements = Vec::new_in(arena);
        let mut uses = Vec::new_in(arena);
        let mut sealings = Vec::new_in(arena);
        let mut mixins = Vec::new_in(arena);
        let mut methods = Vec::new_in(arena);
        let mut properties = Vec::new_in(arena);
        let mut where_constraints = Vec::new_in(arena);
        let mut return_type = Vec::new_in(arena);
        let mut return_entries = Vec::new_in(self.scratch);
        let mut throws = Vec::new_in(arena);
        let mut asserts = Vec::new_in(arena);
        let mut asserts_if_true = Vec::new_in(arena);
        let mut asserts_if_false = Vec::new_in(arena);
        let mut self_out = Vec::new_in(arena);
        let mut pure_unless_callable_impure = Vec::new_in(arena);
        let mut var = Vec::new_in(arena);
        let mut parameters = Vec::new_in(arena);
        let mut parameter_entries = Vec::new_in(self.scratch);
        let mut parameter_outs = Vec::new_in(arena);

        for element in document.into_iter().flat_map(|document| document.elements.iter()) {
            let Element::Tag(tag) = element else { continue };
            let tag = *tag;

            if let Some(marker) = self.marker_annotation_tag(tag) {
                tags.set(marker);

                continue;
            }

            match &tag.value {
                TagValue::TypeAlias(alias) => type_aliases.push(self.lower_type_alias_annotation(alias)),
                TagValue::TypeAliasImport(import) => {
                    imported_type_aliases.push(self.lower_imported_type_alias_annotation(import));
                }
                TagValue::Extends(value) => extends.extend(self.lower_extends_annotation(value)),
                TagValue::Implements(value) => implements.extend(self.lower_implements_annotation(value)),
                TagValue::RequireExtends(value) => require_extends.extend(self.lower_require_extends_annotation(value)),
                TagValue::RequireImplements(value) => {
                    require_implements.extend(self.lower_require_implements_annotation(value));
                }
                TagValue::Use(value) => uses.extend(self.lower_use_annotation(value)),
                TagValue::Mixin(value) => mixins.push(self.lower_mixin_annotation(value)),
                TagValue::Sealed(value) => sealings.push(self.lower_sealed_annotation(value)),
                TagValue::Inheritors(value) => sealings.push(self.lower_inheritors_annotation(value)),
                TagValue::Method(value) => {
                    if let Some(owner) = owner {
                        methods.push(self.lower_method_annotation(value, owner));
                    }
                }
                TagValue::Property(value) => {
                    properties.push(self.lower_property_annotation(value, PropertyAnnotationKind::ReadWrite));
                }
                TagValue::PropertyRead(value) => {
                    properties.push(self.lower_property_annotation(value, PropertyAnnotationKind::Read));
                }
                TagValue::PropertyWrite(value) => {
                    properties.push(self.lower_property_annotation(value, PropertyAnnotationKind::Write));
                }
                TagValue::Where(value) => where_constraints.push(self.lower_where_constraint_annotation(value)),
                TagValue::Return(value) | TagValue::RealReturn(value) => {
                    let precedence = match TagVendor::from_name(tag.name.value) {
                        Some(TagVendor::Psalm) => 2u8,
                        Some(TagVendor::PhpStan) => 1,
                        _ => 0,
                    };

                    return_entries.push((precedence, *self.lower_type_annotation(value.r#type)));
                }
                TagValue::Throws(value) => throws.push(self.lower_throws_annotation(value)),
                TagValue::Assert(value) => asserts.push(self.lower_assert_annotation(value)),
                TagValue::AssertIfTrue(value) => asserts_if_true.push(self.lower_assert_annotation(value)),
                TagValue::AssertIfFalse(value) => asserts_if_false.push(self.lower_assert_annotation(value)),
                TagValue::SelfOut(value) => self_out.push(self.lower_self_out_annotation(value)),
                TagValue::PureUnlessCallableIsImpure(value) => {
                    pure_unless_callable_impure.push(self.phpdoc_variable(&value.parameter))
                }
                TagValue::Var(value) => var.push(VariableAnnotation {
                    span: tag.span(),
                    type_annotation: self.lower_type_annotation(value.r#type),
                    variable: value.variable.as_ref().map(|variable| self.phpdoc_variable(variable)),
                    errors: &[],
                }),
                TagValue::Param(value) => {
                    let precedence = match TagVendor::from_name(tag.name.value) {
                        Some(TagVendor::Psalm) => 2u8,
                        Some(TagVendor::PhpStan) => 1,
                        _ => 0,
                    };

                    parameter_entries.push((
                        precedence,
                        ParameterAnnotation {
                            span: tag.span(),
                            r#type: Some(self.lower_type_annotation(value.r#type)),
                            is_by_reference: value.ampersand.is_some(),
                            is_variadic: value.ellipsis.is_some(),
                            variable: self.phpdoc_variable(&value.parameter),
                            default_value: None,
                        },
                    ));
                }
                TagValue::ParamOut(value) => {
                    parameter_outs.push(ParameterOutAnnotation {
                        span: tag.span(),
                        r#type: self.lower_type_annotation(value.r#type),
                        variable: self.phpdoc_variable(&value.parameter),
                    });
                }
                _ => {}
            }
        }

        parameter_entries.sort_by_key(|(precedence, _)| *precedence);
        parameters.extend(parameter_entries.iter().map(|(_, parameter)| *parameter));

        return_entries.sort_by_key(|(precedence, _)| std::cmp::Reverse(*precedence));
        return_type.extend(return_entries.iter().map(|(_, annotation)| *annotation));

        let no_explicit_asserts = asserts.is_empty() && asserts_if_true.is_empty() && asserts_if_false.is_empty();
        let (asserts_if_true, asserts_if_false, assertions_inferred): (&[_], &[_], bool) = match inferred_assertions {
            Some((if_true, if_false)) if no_explicit_asserts => (if_true, if_false, true),
            _ => (asserts_if_true.leak(), asserts_if_false.leak(), false),
        };

        let inherited_type_parameters =
            self.type_resolution.inherited_templates(self.arena, self.settings.inherit_static_templates);

        if document.is_none() && inherited_type_parameters.is_empty() && !assertions_inferred {
            return (None, false);
        }

        let annotation = arena.alloc(ItemAnnotation {
            span: document.map_or(Span::zero(), |document| document.span),
            type_aliases: type_aliases.leak(),
            imported_type_aliases: imported_type_aliases.leak(),
            type_parameters,
            inherited_type_parameters,
            extends: extends.leak(),
            require_extends: require_extends.leak(),
            implements: implements.leak(),
            require_implements: require_implements.leak(),
            uses: uses.leak(),
            sealings: sealings.leak(),
            mixins: mixins.leak(),
            methods: methods.leak(),
            properties: properties.leak(),
            parameters: parameters.leak(),
            parameter_outs: parameter_outs.leak(),
            where_constraints: where_constraints.leak(),
            return_type: return_type.leak(),
            throws: throws.leak(),
            asserts: asserts.leak(),
            asserts_if_true,
            asserts_if_false,
            self_out: self_out.leak(),
            pure_unless_callable_impure: pure_unless_callable_impure.leak(),
            var: var.leak(),
            tags,
            errors: self.lower_annotation_errors(document),
        });

        (Some(annotation), assertions_inferred)
    }

    pub(crate) fn lower_annotation_errors(&self, document: Option<&Document<'scratch>>) -> &'arena [AnnotationError] {
        match document {
            Some(document) if !document.errors.is_empty() => {
                self.arena.alloc_slice_fill_iter(document.errors.iter().map(|error| AnnotationError::from(*error)))
            }
            _ => &[],
        }
    }

    fn marker_annotation_tag(&self, tag: &Tag<'_>) -> Option<ItemAnnotationTag> {
        Some(match tag.value {
            TagValue::Deprecated(_) => ItemAnnotationTag::Deprecated,
            TagValue::Final(_) => ItemAnnotationTag::Final,
            TagValue::Internal(_) => ItemAnnotationTag::Internal,
            TagValue::Api(_) => ItemAnnotationTag::Api,
            TagValue::Experimental(_) => ItemAnnotationTag::Experimental,
            TagValue::Pure(_) => ItemAnnotationTag::Pure,
            TagValue::Impure(_) => ItemAnnotationTag::Impure,
            TagValue::Readonly(_) => ItemAnnotationTag::Readonly,
            TagValue::MustUse(_) => ItemAnnotationTag::MustUse,
            TagValue::NoNamedArguments(_) => ItemAnnotationTag::NoNamedArguments,
            TagValue::NotDeprecated(_) => ItemAnnotationTag::NotDeprecated,
            TagValue::EnumInterface(_) => ItemAnnotationTag::EnumInterface,
            TagValue::ConsistentConstructor(_) => ItemAnnotationTag::ConsistentConstructor,
            TagValue::ConsistentTemplates(_) => ItemAnnotationTag::ConsistentTypeParameterAnnotations,
            TagValue::SealProperties(_) => ItemAnnotationTag::SealProperties,
            TagValue::NoSealProperties(_) => ItemAnnotationTag::NoSealProperties,
            TagValue::SealMethods(_) => ItemAnnotationTag::SealMethods,
            TagValue::NoSealMethods(_) => ItemAnnotationTag::NoSealMethods,
            TagValue::MutationFree(_) => ItemAnnotationTag::MutationFree,
            TagValue::ExternalMutationFree(_) => ItemAnnotationTag::ExternalMutationFree,
            TagValue::SuspendsFiber(_) => ItemAnnotationTag::SuspendsFiber,
            TagValue::IgnoreNullableReturn(_) => ItemAnnotationTag::IgnoreNullableReturnType,
            TagValue::IgnoreFalsableReturn(_) => ItemAnnotationTag::IgnoreFalsableReturnType,
            TagValue::InheritDoc(_) => ItemAnnotationTag::InheritDoc,
            _ => {
                return None;
            }
        })
    }
}
