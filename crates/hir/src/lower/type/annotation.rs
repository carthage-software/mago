use bumpalo::collections::Vec;

use mago_phpdoc_syntax::cst::Identifier as PHPDocIdentifier;
use mago_phpdoc_syntax::cst::r#type::AliasName;
use mago_phpdoc_syntax::cst::r#type::CallableType;
use mago_phpdoc_syntax::cst::r#type::CallableTypeKind as PHPDocCallableTypeKind;
use mago_phpdoc_syntax::cst::r#type::GenericParameters;
use mago_phpdoc_syntax::cst::r#type::GlobalWildcardSelector;
use mago_phpdoc_syntax::cst::r#type::IntOrKeyword;
use mago_phpdoc_syntax::cst::r#type::MemberReferenceSelector as PHPDocMemberReferenceSelector;
use mago_phpdoc_syntax::cst::r#type::PropertiesOfFilter as PHPDocPropertiesOfFilter;
use mago_phpdoc_syntax::cst::r#type::ReferenceType;
use mago_phpdoc_syntax::cst::r#type::ShapeField;
use mago_phpdoc_syntax::cst::r#type::ShapeKey;
use mago_phpdoc_syntax::cst::r#type::SingleGenericParameter;
use mago_phpdoc_syntax::cst::r#type::Type;
use mago_span::HasSpan;

use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::ir::name::Name;
use crate::ir::r#type::annotation::CallableParameter;
use crate::ir::r#type::annotation::CallableTypeAnnotation;
use crate::ir::r#type::annotation::CallableTypeKind;
use crate::ir::r#type::annotation::ConditionalTypeAnnotation;
use crate::ir::r#type::annotation::FloatLiteral;
use crate::ir::r#type::annotation::GenericParameterAnnotation;
use crate::ir::r#type::annotation::GlobalSelector;
use crate::ir::r#type::annotation::IntLiteral;
use crate::ir::r#type::annotation::MemberReferenceSelector;
use crate::ir::r#type::annotation::NamedTypeAnnotation;
use crate::ir::r#type::annotation::ObjectShapeTypeAnnotation;
use crate::ir::r#type::annotation::PropertiesOfFilter;
use crate::ir::r#type::annotation::ShapeTypeAnnotation;
use crate::ir::r#type::annotation::ShapeTypeAnnotationAdditionalFields;
use crate::ir::r#type::annotation::ShapeTypeAnnotationField;
use crate::ir::r#type::annotation::ShapeTypeAnnotationKey;
use crate::ir::r#type::annotation::StringCasing;
use crate::ir::r#type::annotation::StringLiteral;
use crate::ir::r#type::annotation::StringTypeAnnotation;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::r#type::annotation::TypeAnnotationKind;
use crate::ir::variable::DirectVariable;
use crate::lower::Lowering;
use crate::lower::resolution::namespace::NameResolutionKind;

impl<'arena> Lowering<'arena> {
    pub(crate) fn lower_type_annotation(&self, ty: &'arena Type<'arena>) -> &'arena TypeAnnotation<'arena> {
        self.arena.alloc(TypeAnnotation { span: ty.span(), kind: self.lower_type_annotation_kind(ty) })
    }

    pub(crate) fn lower_type_annotation_kind(&self, ty: &'arena Type<'arena>) -> TypeAnnotationKind<'arena> {
        match ty {
            Type::Parenthesized(parenthesized) => self.lower_type_annotation_kind(parenthesized.inner),
            Type::Union(_) => TypeAnnotationKind::Union(self.collect_union_annotation(ty)),
            Type::Intersection(_) => TypeAnnotationKind::Intersection(self.collect_intersection_annotation(ty)),
            Type::Nullable(nullable) => {
                let inner = self.lower_type_annotation_kind(nullable.inner);

                TypeAnnotationKind::Union(self.arena.alloc_slice_copy(&[inner, TypeAnnotationKind::Null]))
            }
            Type::TrailingPipe(trailing) => self.lower_type_annotation_kind(trailing.inner),
            Type::Array(array) => self.array(false, array.parameters.as_ref()),
            Type::NonEmptyArray(array) => self.array(true, array.parameters.as_ref()),
            Type::AssociativeArray(array) => self.array(false, array.parameters.as_ref()),
            Type::List(list) => TypeAnnotationKind::List(false, self.list_value(list.parameters.as_ref())),
            Type::NonEmptyList(list) => TypeAnnotationKind::List(true, self.list_value(list.parameters.as_ref())),
            Type::Iterable(iterable) => {
                let (key, value) = self.key_value(iterable.parameters.as_ref(), self.alloc_mixed());

                TypeAnnotationKind::Iterable(key, value)
            }
            Type::ClassString(string) => {
                TypeAnnotationKind::ClassString(self.class_like_string_parameter(string.parameter.as_ref()))
            }
            Type::ClassLikeString(string) => {
                TypeAnnotationKind::ClassLikeString(self.class_like_string_parameter(string.parameter.as_ref()))
            }
            Type::InterfaceString(string) => {
                TypeAnnotationKind::InterfaceString(self.class_like_string_parameter(string.parameter.as_ref()))
            }
            Type::EnumString(string) => {
                TypeAnnotationKind::EnumString(self.class_like_string_parameter(string.parameter.as_ref()))
            }
            Type::TraitString(string) => {
                TypeAnnotationKind::TraitString(self.class_like_string_parameter(string.parameter.as_ref()))
            }
            Type::Reference(reference) => self.lower_reference(reference),
            Type::Mixed(_) => TypeAnnotationKind::Mixed(false),
            Type::NonEmptyMixed(_) => TypeAnnotationKind::Mixed(true),
            Type::Null(_) => TypeAnnotationKind::Null,
            Type::Void(_) => TypeAnnotationKind::Void,
            Type::Never(_) => TypeAnnotationKind::Never,
            Type::Resource(_) => TypeAnnotationKind::Resource(None),
            Type::ClosedResource(_) => TypeAnnotationKind::Resource(Some(false)),
            Type::OpenResource(_) => TypeAnnotationKind::Resource(Some(true)),
            Type::True(_) => TypeAnnotationKind::Bool(Some(true)),
            Type::False(_) => TypeAnnotationKind::Bool(Some(false)),
            Type::Bool(_) => TypeAnnotationKind::Bool(None),
            Type::Float(_) => TypeAnnotationKind::Float(None),
            Type::Int(_) => TypeAnnotationKind::Int(None),
            Type::PositiveInt(_) => TypeAnnotationKind::IntRange(Some(1), None),
            Type::NegativeInt(_) => TypeAnnotationKind::IntRange(None, Some(-1)),
            Type::NonPositiveInt(_) => TypeAnnotationKind::IntRange(None, Some(0)),
            Type::NonNegativeInt(_) => TypeAnnotationKind::IntRange(Some(0), None),
            Type::NonZeroInt(_) => TypeAnnotationKind::Union(self.arena.alloc_slice_copy(&[
                TypeAnnotationKind::IntRange(None, Some(-1)),
                TypeAnnotationKind::IntRange(Some(1), None),
            ])),
            Type::String(_) => self.string(None, None, false, false, false, false),
            Type::StringableObject(_) => TypeAnnotationKind::StringableObject,
            Type::ArrayKey(_) => self.array_key_kind(),
            Type::Numeric(_) => TypeAnnotationKind::Numeric,
            Type::Scalar(_) => self.scalar_kind(),
            Type::CallableString(_) => self.string(None, None, false, false, false, true),
            Type::LowercaseCallableString(_) => {
                self.string(Some(StringCasing::Lowercase), None, false, false, false, true)
            }
            Type::UppercaseCallableString(_) => {
                self.string(Some(StringCasing::Uppercase), None, false, false, false, true)
            }
            Type::NumericString(_) => self.string(None, None, false, true, false, false),
            Type::NonEmptyString(_) => self.string(None, None, true, false, false, false),
            Type::NonEmptyLowercaseString(_) => {
                self.string(Some(StringCasing::Lowercase), None, true, false, false, false)
            }
            Type::LowercaseString(_) => self.string(Some(StringCasing::Lowercase), None, false, false, false, false),
            Type::NonEmptyUppercaseString(_) => {
                self.string(Some(StringCasing::Uppercase), None, true, false, false, false)
            }
            Type::UppercaseString(_) => self.string(Some(StringCasing::Uppercase), None, false, false, false, false),
            Type::TruthyString(_) => self.string(None, None, false, false, true, false),
            Type::NonFalsyString(_) => self.string(None, None, false, false, true, false),
            Type::UnspecifiedLiteralInt(_) => TypeAnnotationKind::Int(Some(IntLiteral::Unspecified)),
            Type::UnspecifiedLiteralString(_) => {
                self.string(None, Some(StringLiteral::Unspecified), false, false, false, false)
            }
            Type::UnspecifiedLiteralFloat(_) => TypeAnnotationKind::Float(Some(FloatLiteral::Unspecified)),
            Type::NonEmptyUnspecifiedLiteralString(_) => {
                self.string(None, Some(StringLiteral::Unspecified), true, false, false, false)
            }
            Type::LiteralFloat(literal) => TypeAnnotationKind::Float(Some(FloatLiteral::Specific(literal.value))),
            Type::LiteralInt(literal) => TypeAnnotationKind::Int(Some(IntLiteral::Specific(literal.value as i64))),
            Type::LiteralString(literal) => {
                self.string(None, Some(StringLiteral::Specific(literal.value)), false, false, false, false)
            }

            Type::MemberReference(member) => TypeAnnotationKind::MemberReference(
                self.resolve_phpdoc_class(&member.class),
                self.lower_member_reference_selector(&member.member),
            ),
            Type::AliasReference(alias) => {
                let alias_name = match &alias.alias {
                    AliasName::Identifier(identifier) => self.phpdoc_name(identifier),
                    AliasName::Keyword(keyword) => Name { span: keyword.span, value: keyword.value },
                };

                TypeAnnotationKind::AliasReference(self.resolve_phpdoc_class(&alias.class), alias_name)
            }
            Type::GlobalWildcardReference(global) => {
                let selector = match &global.selector {
                    GlobalWildcardSelector::StartsWith(identifier, _) => {
                        GlobalSelector::StartsWith(self.resolve_phpdoc_class(identifier))
                    }
                    GlobalWildcardSelector::EndsWith(_, identifier) => {
                        GlobalSelector::EndsWith(self.resolve_phpdoc_class(identifier))
                    }
                };

                TypeAnnotationKind::GlobalSelector(selector)
            }
            Type::Object(object) => match &object.properties {
                Some(properties) => TypeAnnotationKind::ObjectShape(ObjectShapeTypeAnnotation {
                    fields: self.lower_shape_fields(properties.fields.as_slice()),
                    sealed: properties.ellipsis.is_none(),
                }),
                None => TypeAnnotationKind::Object,
            },
            Type::Shape(shape) => TypeAnnotationKind::Shape(ShapeTypeAnnotation {
                fields: self.lower_shape_fields(shape.fields.as_slice()),
                additional_fields: match &shape.additional_fields {
                    Some(additional_fields) => {
                        let (key_type, value_type) =
                            self.key_value(additional_fields.parameters.as_ref(), self.alloc_array_key());

                        Some(ShapeTypeAnnotationAdditionalFields { key_type, value_type })
                    }
                    None => None,
                },
            }),
            Type::Callable(callable) => self.lower_callable(callable),
            Type::Variable(variable) => {
                TypeAnnotationKind::Variable(DirectVariable { span: variable.span, name: variable.value })
            }
            Type::Conditional(conditional) => TypeAnnotationKind::Conditional(ConditionalTypeAnnotation {
                subject: self.alloc_type_annotation_kind(conditional.subject),
                target: self.alloc_type_annotation_kind(conditional.target),
                is_negated: conditional.not.is_some(),
                then: self.alloc_type_annotation_kind(conditional.then),
                r#else: self.alloc_type_annotation_kind(conditional.r#else),
            }),
            Type::KeyOf(key_of) => TypeAnnotationKind::KeyOf(self.lower_single_parameter(&key_of.parameter)),
            Type::ValueOf(value_of) => TypeAnnotationKind::ValueOf(self.lower_single_parameter(&value_of.parameter)),
            Type::IntMask(int_mask) => {
                TypeAnnotationKind::IntMask(self.lower_generic_parameters(Some(&int_mask.parameters)))
            }
            Type::IntMaskOf(int_mask_of) => {
                TypeAnnotationKind::IntMaskOf(self.lower_single_parameter(&int_mask_of.parameter))
            }
            Type::New(new) => TypeAnnotationKind::New(self.lower_single_parameter(&new.parameter)),
            Type::TemplateType(template) => {
                TypeAnnotationKind::TemplateType(self.lower_generic_parameters(Some(&template.parameters)))
            }
            Type::IndexAccess(index_access) => TypeAnnotationKind::IndexAccess(
                self.alloc_type_annotation_kind(index_access.target),
                self.alloc_type_annotation_kind(index_access.index),
            ),
            Type::Negated(negated) => TypeAnnotationKind::Negated(self.alloc_type_annotation_kind(negated.operand)),
            Type::Posited(posited) => TypeAnnotationKind::Posited(self.alloc_type_annotation_kind(posited.operand)),
            Type::IntRange(range) => {
                TypeAnnotationKind::IntRange(self.int_bound(&range.min), self.int_bound(&range.max))
            }
            Type::PropertiesOf(properties_of) => TypeAnnotationKind::PropertiesOf(
                self.lower_properties_filter(&properties_of.filter),
                self.lower_single_parameter(&properties_of.parameter),
            ),
            Type::Slice(slice) => TypeAnnotationKind::Slice(self.alloc_type_annotation_kind(slice.inner)),
            Type::Wildcard(_) => TypeAnnotationKind::Wildcard,

            _ => {
                debug_assert!(false, "unhandled type annotation kind: {ty:?}");

                // SAFETY: every non-`#[non_exhaustive]` variant of `Type` is handled above; the
                // debug assertion catches any future variant during development so it can be mapped here.
                unsafe { std::hint::unreachable_unchecked() }
            }
        }
    }

    fn alloc_type_annotation_kind(&self, ty: &'arena Type<'arena>) -> &'arena TypeAnnotationKind<'arena> {
        self.arena.alloc(self.lower_type_annotation_kind(ty))
    }

    fn collect_union_annotation(&self, ty: &'arena Type<'arena>) -> &'arena [TypeAnnotationKind<'arena>] {
        let mut members = Vec::new_in(self.arena);
        self.flatten_union_annotation(ty, &mut members);

        members.into_bump_slice()
    }

    fn flatten_union_annotation(
        &self,
        ty: &'arena Type<'arena>,
        members: &mut Vec<'arena, TypeAnnotationKind<'arena>>,
    ) {
        match ty {
            Type::Union(union) => {
                self.flatten_union_annotation(union.left, members);
                self.flatten_union_annotation(union.right, members);
            }
            Type::Parenthesized(parenthesized) => self.flatten_union_annotation(parenthesized.inner, members),
            Type::TrailingPipe(trailing) => self.flatten_union_annotation(trailing.inner, members),
            _ => members.push(self.lower_type_annotation_kind(ty)),
        }
    }

    fn collect_intersection_annotation(&self, ty: &'arena Type<'arena>) -> &'arena [TypeAnnotationKind<'arena>] {
        let mut members = Vec::new_in(self.arena);
        self.flatten_intersection_annotation(ty, &mut members);

        members.into_bump_slice()
    }

    fn flatten_intersection_annotation(
        &self,
        ty: &'arena Type<'arena>,
        members: &mut Vec<'arena, TypeAnnotationKind<'arena>>,
    ) {
        match ty {
            Type::Intersection(intersection) => {
                self.flatten_intersection_annotation(intersection.left, members);
                self.flatten_intersection_annotation(intersection.right, members);
            }
            Type::Parenthesized(parenthesized) => {
                self.flatten_intersection_annotation(parenthesized.inner, members);
            }
            _ => members.push(self.lower_type_annotation_kind(ty)),
        }
    }

    fn lower_reference(&self, reference: &'arena ReferenceType<'arena>) -> TypeAnnotationKind<'arena> {
        let identifier = &reference.identifier;
        let value = identifier.value;

        if reference.parameters.is_none() {
            if let Some((defining_entity, bound)) = self.type_resolution.lookup_template(value) {
                let name = Name { span: identifier.span, value };

                return TypeAnnotationKind::GenericParameter(GenericParameterAnnotation {
                    name,
                    defining_entity,
                    bound,
                });
            }

            if let Some((source_class, alias_name)) = self.type_resolution.lookup_alias(value) {
                return TypeAnnotationKind::AliasReference(source_class, alias_name);
            }
        }

        TypeAnnotationKind::Named(NamedTypeAnnotation {
            name: self.resolve_phpdoc_class(identifier),
            type_arguments: self.lower_generic_parameters(reference.parameters.as_ref()),
        })
    }

    pub(crate) fn resolve_phpdoc_class(&self, identifier: &PHPDocIdentifier<'arena>) -> Identifier<'arena> {
        self.resolve_phpdoc_identifier(identifier, NameResolutionKind::Default)
    }

    pub(crate) fn resolve_phpdoc_identifier(
        &self,
        identifier: &PHPDocIdentifier<'arena>,
        resolution: NameResolutionKind,
    ) -> Identifier<'arena> {
        let value = identifier.value;
        let kind = if let [b'\\', ..] = value {
            IdentifierKind::FullyQualified
        } else if memchr::memchr(b'\\', value).is_some() {
            IdentifierKind::Qualified
        } else {
            IdentifierKind::Local
        };

        Identifier { span: identifier.span, value: self.namespace_resolution.resolve_name(resolution, value), kind }
    }

    fn array(
        &self,
        non_empty: bool,
        parameters: Option<&'arena GenericParameters<'arena>>,
    ) -> TypeAnnotationKind<'arena> {
        let (key, value) = self.key_value(parameters, self.alloc_array_key());

        TypeAnnotationKind::Array(non_empty, key, value)
    }

    fn key_value(
        &self,
        parameters: Option<&'arena GenericParameters<'arena>>,
        default_key: &'arena TypeAnnotationKind<'arena>,
    ) -> (&'arena TypeAnnotationKind<'arena>, &'arena TypeAnnotationKind<'arena>) {
        match self.lower_generic_parameters(parameters) {
            [] => (default_key, self.alloc_mixed()),
            [value] => (default_key, value),
            [key, value, ..] => (key, value),
        }
    }

    fn list_value(&self, parameters: Option<&'arena GenericParameters<'arena>>) -> &'arena TypeAnnotationKind<'arena> {
        match self.lower_generic_parameters(parameters) {
            [] => self.alloc_mixed(),
            [value, ..] => value,
        }
    }

    fn lower_generic_parameters(
        &self,
        parameters: Option<&'arena GenericParameters<'arena>>,
    ) -> &'arena [TypeAnnotationKind<'arena>] {
        match parameters {
            Some(parameters) => self.arena.alloc_slice_fill_iter(
                parameters.entries.iter().map(|entry| self.lower_type_annotation_kind(&entry.inner)),
            ),
            None => &[],
        }
    }

    fn lower_single_parameter(
        &self,
        parameter: &'arena SingleGenericParameter<'arena>,
    ) -> &'arena TypeAnnotationKind<'arena> {
        self.alloc_type_annotation_kind(&parameter.entry.inner)
    }

    fn class_like_string_parameter(
        &self,
        parameter: Option<&'arena SingleGenericParameter<'arena>>,
    ) -> &'arena TypeAnnotationKind<'arena> {
        match parameter {
            Some(parameter) => self.lower_single_parameter(parameter),
            None => self.arena.alloc(TypeAnnotationKind::Wildcard),
        }
    }

    fn alloc_mixed(&self) -> &'arena TypeAnnotationKind<'arena> {
        self.arena.alloc(TypeAnnotationKind::Mixed(false))
    }

    fn alloc_array_key(&self) -> &'arena TypeAnnotationKind<'arena> {
        self.arena.alloc(self.array_key_kind())
    }

    fn array_key_kind(&self) -> TypeAnnotationKind<'arena> {
        TypeAnnotationKind::Union(
            self.arena.alloc_slice_copy(&[
                TypeAnnotationKind::Int(None),
                self.string(None, None, false, false, false, false),
            ]),
        )
    }

    fn scalar_kind(&self) -> TypeAnnotationKind<'arena> {
        TypeAnnotationKind::Union(self.arena.alloc_slice_copy(&[
            TypeAnnotationKind::Int(None),
            TypeAnnotationKind::Float(None),
            self.string(None, None, false, false, false, false),
            TypeAnnotationKind::Bool(None),
        ]))
    }

    fn string(
        &self,
        casing: Option<StringCasing>,
        literal: Option<StringLiteral<'arena>>,
        non_empty: bool,
        numeric: bool,
        truthy: bool,
        callable: bool,
    ) -> TypeAnnotationKind<'arena> {
        TypeAnnotationKind::String(StringTypeAnnotation { casing, literal, non_empty, numeric, truthy, callable })
    }

    fn lower_shape_fields(&self, fields: &'arena [ShapeField<'arena>]) -> &'arena [ShapeTypeAnnotationField<'arena>] {
        self.arena.alloc_slice_fill_iter(fields.iter().enumerate().map(|(index, field)| {
            let (key, optional) = match &field.key {
                Some(field_key) => (self.lower_shape_key(&field_key.key), field_key.question_mark.is_some()),
                None => (ShapeTypeAnnotationKey::Integer(index as i64), false),
            };

            ShapeTypeAnnotationField { key, optional, value: self.lower_type_annotation_kind(field.value) }
        }))
    }

    fn lower_shape_key(&self, key: &ShapeKey<'arena>) -> ShapeTypeAnnotationKey<'arena> {
        match key {
            ShapeKey::String { value, .. } => ShapeTypeAnnotationKey::String(value),
            ShapeKey::Integer { value, .. } => ShapeTypeAnnotationKey::Integer(*value),
            ShapeKey::ClassLikeConstant { class_name, constant_name, .. } => ShapeTypeAnnotationKey::ClassLikeConstant(
                self.resolve_phpdoc_class(class_name),
                self.phpdoc_name(constant_name),
            ),
        }
    }

    fn lower_callable(&self, callable: &'arena CallableType<'arena>) -> TypeAnnotationKind<'arena> {
        let kind = match callable.kind {
            PHPDocCallableTypeKind::Callable => CallableTypeKind::Callable,
            PHPDocCallableTypeKind::PureCallable => CallableTypeKind::PureCallable,
            PHPDocCallableTypeKind::Closure => CallableTypeKind::Closure,
            PHPDocCallableTypeKind::PureClosure => CallableTypeKind::PureClosure,
        };

        let parameters: &[CallableParameter<'arena>] = match &callable.specification {
            Some(specification) => {
                self.arena.alloc_slice_fill_iter(specification.parameters.entries.iter().map(|parameter| {
                    CallableParameter {
                        r#type: parameter
                            .parameter_type
                            .as_ref()
                            .map(|parameter_type| self.lower_type_annotation(parameter_type)),
                        variadic: parameter.ellipsis.is_some(),
                        by_reference: parameter.ampersand.is_some(),
                        variable: parameter
                            .variable
                            .as_ref()
                            .map(|variable| DirectVariable { span: variable.span, name: variable.value }),
                        has_default: parameter.equals.is_some(),
                    }
                }))
            }
            None => &[],
        };

        let r#return = callable.specification.as_ref().and_then(|specification| {
            specification
                .return_type
                .as_ref()
                .map(|return_type| self.alloc_type_annotation_kind(return_type.return_type))
        });

        TypeAnnotationKind::Callable(CallableTypeAnnotation { kind, parameters, r#return })
    }

    fn lower_member_reference_selector(
        &self,
        selector: &PHPDocMemberReferenceSelector<'arena>,
    ) -> MemberReferenceSelector<'arena> {
        match selector {
            PHPDocMemberReferenceSelector::Wildcard(_) => MemberReferenceSelector::Wildcard,
            PHPDocMemberReferenceSelector::Identifier(identifier) => {
                MemberReferenceSelector::Exact(self.phpdoc_name(identifier))
            }
            PHPDocMemberReferenceSelector::StartsWith(identifier, _) => {
                MemberReferenceSelector::StartsWith(self.phpdoc_name(identifier))
            }
            PHPDocMemberReferenceSelector::EndsWith(_, identifier) => {
                MemberReferenceSelector::EndsWith(self.phpdoc_name(identifier))
            }
        }
    }

    fn lower_properties_filter(&self, filter: &PHPDocPropertiesOfFilter) -> PropertiesOfFilter {
        match filter {
            PHPDocPropertiesOfFilter::All => PropertiesOfFilter::All,
            PHPDocPropertiesOfFilter::Public => PropertiesOfFilter::Public,
            PHPDocPropertiesOfFilter::Protected => PropertiesOfFilter::Protected,
            PHPDocPropertiesOfFilter::Private => PropertiesOfFilter::Private,
        }
    }

    fn int_bound(&self, value: &IntOrKeyword<'arena>) -> Option<i64> {
        match value {
            IntOrKeyword::Int(literal) => Some(literal.value as i64),
            IntOrKeyword::NegativeInt { int, .. } => Some(-(int.value as i64)),
            IntOrKeyword::Keyword(_) => None,
        }
    }

    pub(crate) fn phpdoc_name(&self, identifier: &PHPDocIdentifier<'arena>) -> Name<'arena> {
        Name { span: identifier.span, value: identifier.value }
    }

    pub(crate) fn lower_named_type(&self, ty: &'arena Type<'arena>) -> Option<NamedTypeAnnotation<'arena>> {
        match ty {
            Type::Parenthesized(parenthesized) => self.lower_named_type(parenthesized.inner),
            Type::Reference(reference) => Some(NamedTypeAnnotation {
                name: self.resolve_phpdoc_class(&reference.identifier),
                type_arguments: self.lower_generic_parameters(reference.parameters.as_ref()),
            }),
            _ => None,
        }
    }

    pub(crate) fn lower_named_types(&self, ty: &'arena Type<'arena>) -> &'arena [NamedTypeAnnotation<'arena>] {
        let mut members = Vec::new_in(self.arena);
        self.flatten_named_types(ty, &mut members);

        members.into_bump_slice()
    }

    fn flatten_named_types(&self, ty: &'arena Type<'arena>, members: &mut Vec<'arena, NamedTypeAnnotation<'arena>>) {
        match ty {
            Type::Union(union) => {
                self.flatten_named_types(union.left, members);
                self.flatten_named_types(union.right, members);
            }
            Type::Parenthesized(parenthesized) => self.flatten_named_types(parenthesized.inner, members),
            _ => {
                if let Some(named) = self.lower_named_type(ty) {
                    members.push(named);
                }
            }
        }
    }
}
