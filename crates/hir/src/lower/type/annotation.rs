use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_phpdoc_syntax::cst::Identifier as PHPDocIdentifier;
use mago_phpdoc_syntax::cst::Variable as PHPDocVariable;
use mago_phpdoc_syntax::cst::r#type::AliasName;
use mago_phpdoc_syntax::cst::r#type::CallableType;
use mago_phpdoc_syntax::cst::r#type::CallableTypeKind as PHPDocCallableTypeKind;
use mago_phpdoc_syntax::cst::r#type::GenericParameters;
use mago_phpdoc_syntax::cst::r#type::GlobalWildcardSelector;
use mago_phpdoc_syntax::cst::r#type::IntOrKeyword;
use mago_phpdoc_syntax::cst::r#type::MemberReferenceSelector as PHPDocMemberReferenceSelector;
use mago_phpdoc_syntax::cst::r#type::PropertiesOfFilter as PHPDocPropertiesOfFilter;
use mago_phpdoc_syntax::cst::r#type::ReferenceKind as PHPDocReferenceKind;
use mago_phpdoc_syntax::cst::r#type::ReferenceType;
use mago_phpdoc_syntax::cst::r#type::ShapeField;
use mago_phpdoc_syntax::cst::r#type::ShapeKey;
use mago_phpdoc_syntax::cst::r#type::SingleGenericParameter;
use mago_phpdoc_syntax::cst::r#type::Type;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;
use crate::ir::identifier::IdentifierKind;
use crate::ir::name::Name;
use crate::ir::r#type::annotation::CallableTypeAnnotation;
use crate::ir::r#type::annotation::CallableTypeAnnotationParameter;
use crate::ir::r#type::annotation::CallableTypeKind;
use crate::ir::r#type::annotation::ConditionalTypeAnnotation;
use crate::ir::r#type::annotation::FloatLiteral;
use crate::ir::r#type::annotation::GenericParameterTypeAnnotation;
use crate::ir::r#type::annotation::GlobalSelector;
use crate::ir::r#type::annotation::IntLiteral;
use crate::ir::r#type::annotation::MemberReferenceSelector;
use crate::ir::r#type::annotation::NamedTypeAnnotation;
use crate::ir::r#type::annotation::ObjectShapeTypeAnnotation;
use crate::ir::r#type::annotation::PropertiesOfFilter;
use crate::ir::r#type::annotation::ReferenceKind;
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

impl<'scratch, 'arena, S, A> Lowering<'_, 'scratch, 'arena, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_type_annotation(&mut self, ty: &'scratch Type<'scratch>) -> &'arena TypeAnnotation<'arena> {
        self.arena.alloc(TypeAnnotation { span: ty.span(), kind: self.lower_type_annotation_kind(ty) })
    }

    pub(crate) fn lower_type_annotation_kind(&mut self, ty: &'scratch Type<'scratch>) -> TypeAnnotationKind<'arena> {
        match ty {
            Type::Parenthesized(parenthesized) => self.lower_type_annotation_kind(parenthesized.inner),
            Type::Union(_) => TypeAnnotationKind::Union(self.collect_union_annotation(ty)),
            Type::Intersection(_) => TypeAnnotationKind::Intersection(self.collect_intersection_annotation(ty)),
            Type::Nullable(nullable) => {
                let inner = TypeAnnotation {
                    span: nullable.inner.span(),
                    kind: self.lower_type_annotation_kind(nullable.inner),
                };
                let null = TypeAnnotation { span: nullable.question_mark, kind: TypeAnnotationKind::Null };

                TypeAnnotationKind::Union(self.arena.alloc_slice_copy(&[inner, null]))
            }
            Type::TrailingPipe(trailing) => self.lower_type_annotation_kind(trailing.inner),
            Type::Array(array) => self.array(false, array.parameters.as_ref(), ty.span()),
            Type::NonEmptyArray(array) => self.array(true, array.parameters.as_ref(), ty.span()),
            Type::AssociativeArray(array) => self.array(false, array.parameters.as_ref(), ty.span()),
            Type::List(list) => TypeAnnotationKind::List(false, self.list_value(list.parameters.as_ref(), ty.span())),
            Type::NonEmptyList(list) => {
                TypeAnnotationKind::List(true, self.list_value(list.parameters.as_ref(), ty.span()))
            }
            Type::Iterable(iterable) => {
                let (key, value) = self.key_value(iterable.parameters.as_ref(), self.alloc_mixed(ty.span()), ty.span());

                TypeAnnotationKind::Iterable(key, value)
            }
            Type::ClassString(string) => {
                TypeAnnotationKind::ClassString(self.class_like_string_parameter(string.parameter.as_ref(), ty.span()))
            }
            Type::ClassLikeString(string) => TypeAnnotationKind::ClassLikeString(
                self.class_like_string_parameter(string.parameter.as_ref(), ty.span()),
            ),
            Type::InterfaceString(string) => TypeAnnotationKind::InterfaceString(
                self.class_like_string_parameter(string.parameter.as_ref(), ty.span()),
            ),
            Type::EnumString(string) => {
                TypeAnnotationKind::EnumString(self.class_like_string_parameter(string.parameter.as_ref(), ty.span()))
            }
            Type::TraitString(string) => {
                TypeAnnotationKind::TraitString(self.class_like_string_parameter(string.parameter.as_ref(), ty.span()))
            }
            Type::Reference(reference) => self.lower_reference(reference),
            Type::Mixed(_) => TypeAnnotationKind::Mixed(false),
            Type::NonEmptyMixed(_) => TypeAnnotationKind::Mixed(true),
            Type::Null(_) => TypeAnnotationKind::Null,
            Type::Void(_) => TypeAnnotationKind::Void,
            Type::Never(_) => TypeAnnotationKind::Never,
            Type::Resource(_) => TypeAnnotationKind::Resource(None),
            Type::ClosedResource(_) => TypeAnnotationKind::Resource(Some(true)),
            Type::OpenResource(_) => TypeAnnotationKind::Resource(Some(false)),
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
                TypeAnnotation { span: ty.span(), kind: TypeAnnotationKind::IntRange(None, Some(-1)) },
                TypeAnnotation { span: ty.span(), kind: TypeAnnotationKind::IntRange(Some(1), None) },
            ])),
            Type::String(_) => self.string(ty.span(), None, None, false, false, false, false),
            Type::StringableObject(_) => TypeAnnotationKind::StringableObject,
            Type::ArrayKey(_) => self.array_key_kind(),
            Type::Numeric(_) => TypeAnnotationKind::Numeric,
            Type::Scalar(_) => self.scalar_kind(),
            Type::Empty(_) => TypeAnnotationKind::Empty,
            Type::EmptyScalar(_) => TypeAnnotationKind::EmptyScalar,
            Type::CallableString(_) => self.string(ty.span(), None, None, false, false, false, true),
            Type::LowercaseCallableString(_) => {
                self.string(ty.span(), Some(StringCasing::Lowercase), None, false, false, false, true)
            }
            Type::UppercaseCallableString(_) => {
                self.string(ty.span(), Some(StringCasing::Uppercase), None, false, false, false, true)
            }
            Type::NumericString(_) => self.string(ty.span(), None, None, false, true, false, false),
            Type::NonEmptyString(_) => self.string(ty.span(), None, None, true, false, false, false),
            Type::NonEmptyLowercaseString(_) => {
                self.string(ty.span(), Some(StringCasing::Lowercase), None, true, false, false, false)
            }
            Type::LowercaseString(_) => {
                self.string(ty.span(), Some(StringCasing::Lowercase), None, false, false, false, false)
            }
            Type::NonEmptyUppercaseString(_) => {
                self.string(ty.span(), Some(StringCasing::Uppercase), None, true, false, false, false)
            }
            Type::UppercaseString(_) => {
                self.string(ty.span(), Some(StringCasing::Uppercase), None, false, false, false, false)
            }
            Type::TruthyString(_) => self.string(ty.span(), None, None, false, false, true, false),
            Type::NonFalsyString(_) => self.string(ty.span(), None, None, false, false, true, false),
            Type::UnspecifiedLiteralInt(_) => TypeAnnotationKind::Int(Some(IntLiteral::Unspecified)),
            Type::UnspecifiedLiteralString(_) => {
                self.string(ty.span(), None, Some(StringLiteral::Unspecified), false, false, false, false)
            }
            Type::UnspecifiedLiteralFloat(_) => TypeAnnotationKind::Float(Some(FloatLiteral::Unspecified)),
            Type::NonEmptyUnspecifiedLiteralString(_) => {
                self.string(ty.span(), None, Some(StringLiteral::Unspecified), true, false, false, false)
            }
            Type::LiteralFloat(literal) => TypeAnnotationKind::Float(Some(FloatLiteral::Specific(literal.value))),
            Type::LiteralInt(literal) => TypeAnnotationKind::Int(Some(IntLiteral::Specific(literal.value as i64))),
            Type::LiteralString(literal) => {
                let lit = self.interner.intern(literal.value);

                self.string(ty.span(), None, Some(StringLiteral::Specific(lit)), false, false, false, false)
            }
            Type::MemberReference(member) => TypeAnnotationKind::MemberReference(
                self.resolve_member_reference_class(member.kind),
                self.lower_member_reference_selector(&member.member),
            ),
            Type::AliasReference(alias) => {
                let alias_name = match &alias.alias {
                    AliasName::Identifier(identifier) => self.phpdoc_name(identifier),
                    AliasName::Keyword(keyword) => {
                        Name { span: keyword.span, value: self.interner.intern(keyword.value) }
                    }
                };

                TypeAnnotationKind::AliasReference(self.lower_reference_kind(alias.class), alias_name)
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
                    span: object.span(),
                    fields: self.lower_shape_fields(
                        properties.left_brace.join(properties.right_brace),
                        properties.fields.as_slice(),
                    ),
                    sealed: properties.ellipsis.is_none(),
                }),
                None => TypeAnnotationKind::Object,
            },
            Type::Shape(shape) => TypeAnnotationKind::Shape(ShapeTypeAnnotation {
                span: shape.span(),
                fields: self.lower_shape_fields(shape.left_brace.join(shape.right_brace), shape.fields.as_slice()),
                additional_fields: match &shape.additional_fields {
                    Some(additional_fields) => {
                        let (key_type, value_type) = self.shape_additional_key_value(
                            additional_fields.parameters.as_ref(),
                            additional_fields.span(),
                        );

                        Some(ShapeTypeAnnotationAdditionalFields {
                            span: additional_fields.span(),
                            key_type,
                            value_type,
                        })
                    }
                    None => None,
                },
                is_list: shape.kind.is_list(),
                non_empty: shape.kind.is_non_empty(),
            }),
            Type::Callable(callable) => self.lower_callable(callable),
            Type::Variable(variable) => TypeAnnotationKind::Variable(self.phpdoc_variable(variable)),
            Type::ThisVariable(_) => TypeAnnotationKind::ThisVariable,
            Type::Conditional(conditional) => TypeAnnotationKind::Conditional(ConditionalTypeAnnotation {
                span: conditional.span(),
                subject: self.lower_type_annotation(conditional.subject),
                target: self.lower_type_annotation(conditional.target),
                is_negated: conditional.not.is_some(),
                then: self.lower_type_annotation(conditional.then),
                r#else: self.lower_type_annotation(conditional.r#else),
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
                self.lower_type_annotation(index_access.target),
                self.lower_type_annotation(index_access.index),
            ),
            Type::Negated(negated) => TypeAnnotationKind::Negated(self.lower_type_annotation(negated.operand)),
            Type::Posited(posited) => TypeAnnotationKind::Posited(self.lower_type_annotation(posited.operand)),
            Type::IntRange(range) => {
                TypeAnnotationKind::IntRange(self.int_bound(&range.min), self.int_bound(&range.max))
            }
            Type::PropertiesOf(properties_of) => TypeAnnotationKind::PropertiesOf(
                self.lower_properties_filter(&properties_of.filter),
                self.lower_single_parameter(&properties_of.parameter),
            ),
            Type::Slice(slice) => TypeAnnotationKind::Slice(self.lower_type_annotation(slice.inner)),
            Type::Wildcard(_) => TypeAnnotationKind::Wildcard,
            _ => {
                debug_assert!(false, "unhandled type annotation kind: {ty:?}");

                // SAFETY: every non-`#[non_exhaustive]` variant of `Type` is handled above; the
                // debug assertion catches any future variant during development so it can be mapped here.
                unsafe { std::hint::unreachable_unchecked() }
            }
        }
    }

    fn collect_union_annotation(&mut self, ty: &'scratch Type<'scratch>) -> &'arena [TypeAnnotation<'arena>] {
        let mut members = Vec::new_in(self.arena);
        self.flatten_union_annotation(ty, &mut members);

        members.leak()
    }

    fn flatten_union_annotation(
        &mut self,
        ty: &'scratch Type<'scratch>,
        members: &mut Vec<'arena, TypeAnnotation<'arena>, A>,
    ) {
        match ty {
            Type::Union(union) => {
                self.flatten_union_annotation(union.left, members);
                self.flatten_union_annotation(union.right, members);
            }
            Type::Parenthesized(parenthesized) => self.flatten_union_annotation(parenthesized.inner, members),
            Type::TrailingPipe(trailing) => self.flatten_union_annotation(trailing.inner, members),
            _ => members.push(TypeAnnotation { span: ty.span(), kind: self.lower_type_annotation_kind(ty) }),
        }
    }

    fn collect_intersection_annotation(&mut self, ty: &'scratch Type<'scratch>) -> &'arena [TypeAnnotation<'arena>] {
        let mut members = Vec::new_in(self.arena);
        self.flatten_intersection_annotation(ty, &mut members);

        members.leak()
    }

    fn flatten_intersection_annotation(
        &mut self,
        ty: &'scratch Type<'scratch>,
        members: &mut Vec<'arena, TypeAnnotation<'arena>, A>,
    ) {
        match ty {
            Type::Intersection(intersection) => {
                self.flatten_intersection_annotation(intersection.left, members);
                self.flatten_intersection_annotation(intersection.right, members);
            }
            Type::Parenthesized(parenthesized) => {
                self.flatten_intersection_annotation(parenthesized.inner, members);
            }
            _ => members.push(TypeAnnotation { span: ty.span(), kind: self.lower_type_annotation_kind(ty) }),
        }
    }

    fn lower_reference(&mut self, reference: &'scratch ReferenceType<'scratch>) -> TypeAnnotationKind<'arena> {
        if let PHPDocReferenceKind::Identifier(identifier) = reference.kind
            && reference.parameters.is_none()
        {
            let value = identifier.value;

            if let Some((defining_entity, bound)) = self.type_resolution.lookup_template(value) {
                let name = Name { span: identifier.span, value: self.interner.intern(value) };

                return TypeAnnotationKind::GenericParameter(GenericParameterTypeAnnotation {
                    span: reference.span(),
                    name,
                    defining_entity,
                    bound,
                });
            }

            if let Some((source_class, alias_name)) = self.type_resolution.lookup_alias(value) {
                return TypeAnnotationKind::AliasReference(ReferenceKind::Identifier(source_class), alias_name);
            }
        }

        TypeAnnotationKind::Named(self.named_from_reference_kind(reference.kind, reference.parameters.as_ref()))
    }

    fn named_from_reference_kind(
        &mut self,
        kind: PHPDocReferenceKind<'scratch>,
        parameters: Option<&'scratch GenericParameters<'scratch>>,
    ) -> NamedTypeAnnotation<'arena> {
        let span = match parameters {
            Some(parameters) => kind.span().join(parameters.span()),
            None => kind.span(),
        };

        NamedTypeAnnotation {
            span,
            kind: self.lower_reference_kind(kind),
            type_arguments: self.lower_reference_type_arguments(parameters),
        }
    }

    fn lower_reference_kind(&mut self, kind: PHPDocReferenceKind<'scratch>) -> ReferenceKind<'arena> {
        match kind {
            PHPDocReferenceKind::Identifier(identifier) => {
                ReferenceKind::Identifier(self.resolve_phpdoc_class(&identifier))
            }
            PHPDocReferenceKind::Self_(keyword) => ReferenceKind::Self_(self.enclosing_class_or_static(keyword.span)),
            PHPDocReferenceKind::Static(keyword) => ReferenceKind::Static(self.enclosing_class_or_static(keyword.span)),
            PHPDocReferenceKind::Parent(keyword) => {
                ReferenceKind::Parent(Identifier { span: keyword.span, value: b"parent", kind: IdentifierKind::Local })
            }
        }
    }

    fn lower_reference_type_arguments(
        &mut self,
        parameters: Option<&'scratch GenericParameters<'scratch>>,
    ) -> Option<Delimited<'arena, TypeAnnotation<'arena>>> {
        parameters.map(|parameters| Delimited {
            span: parameters.less_than.join(parameters.greater_than),
            items: self.arena.alloc_slice_fill_iter(parameters.entries.iter().map(|entry| TypeAnnotation {
                span: entry.inner.span(),
                kind: self.lower_reference_type_argument(&entry.inner),
            })),
        })
    }

    fn lower_reference_type_argument(&mut self, ty: &'scratch Type<'scratch>) -> TypeAnnotationKind<'arena> {
        match ty {
            Type::ThisVariable(this_variable) => TypeAnnotationKind::Named(NamedTypeAnnotation {
                span: this_variable.span,
                kind: ReferenceKind::Static(self.enclosing_class_or_static(this_variable.span)),
                type_arguments: None,
            }),
            Type::Parenthesized(parenthesized) => self.lower_reference_type_argument(parenthesized.inner),
            _ => self.lower_type_annotation_kind(ty),
        }
    }

    fn resolve_member_reference_class(&mut self, kind: PHPDocReferenceKind<'scratch>) -> Identifier<'arena> {
        match kind {
            PHPDocReferenceKind::Parent(keyword) => {
                Identifier { span: keyword.span, value: b"parent", kind: IdentifierKind::Local }
            }
            PHPDocReferenceKind::Self_(keyword) | PHPDocReferenceKind::Static(keyword) => {
                match self.type_resolution.enclosing_class() {
                    Some(class) => Identifier {
                        span: class.span,
                        value: {
                            let mut lowercased = Vec::new_in(self.scratch);
                            lowercased.extend(class.value.iter().map(u8::to_ascii_lowercase));

                            self.interner.intern(&lowercased)
                        },
                        kind: class.kind,
                    },
                    None => Identifier {
                        span: keyword.span,
                        value: self.interner.intern(keyword.value),
                        kind: IdentifierKind::Local,
                    },
                }
            }
            PHPDocReferenceKind::Identifier(identifier) => self.resolve_phpdoc_class(&identifier),
        }
    }

    pub(crate) fn resolve_phpdoc_class(&mut self, identifier: &PHPDocIdentifier<'scratch>) -> Identifier<'arena> {
        self.resolve_phpdoc_identifier(identifier, NameResolutionKind::Default)
    }

    pub(crate) fn resolve_phpdoc_identifier(
        &mut self,
        identifier: &PHPDocIdentifier<'scratch>,
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

        Identifier {
            span: identifier.span,
            value: self.interner.intern(self.namespace_resolution.resolve_name(resolution, value)),
            kind,
        }
    }

    fn array(
        &mut self,
        non_empty: bool,
        parameters: Option<&'scratch GenericParameters<'scratch>>,
        span: Span,
    ) -> TypeAnnotationKind<'arena> {
        let (key, value) = self.key_value(parameters, self.alloc_array_key(span), span);

        TypeAnnotationKind::Array(non_empty, key, value)
    }

    fn key_value(
        &mut self,
        parameters: Option<&'scratch GenericParameters<'scratch>>,
        default_key: &'arena TypeAnnotation<'arena>,
        span: Span,
    ) -> (&'arena TypeAnnotation<'arena>, &'arena TypeAnnotation<'arena>) {
        match self.lower_generic_parameters(parameters) {
            [] => (default_key, self.alloc_mixed(span)),
            [value] => (default_key, value),
            [key, value, ..] => (key, value),
        }
    }

    fn shape_additional_key_value(
        &mut self,
        parameters: Option<&'scratch GenericParameters<'scratch>>,
        span: Span,
    ) -> (&'arena TypeAnnotation<'arena>, &'arena TypeAnnotation<'arena>) {
        match parameters {
            None => (self.alloc_array_key(span), self.alloc_mixed(span)),
            Some(_) => match self.lower_generic_parameters(parameters) {
                [] => (self.alloc_mixed(span), self.alloc_mixed(span)),
                [key] => (key, self.alloc_mixed(span)),
                [key, value, ..] => (key, value),
            },
        }
    }

    fn list_value(
        &mut self,
        parameters: Option<&'scratch GenericParameters<'scratch>>,
        span: Span,
    ) -> &'arena TypeAnnotation<'arena> {
        match self.lower_generic_parameters(parameters) {
            [] => self.alloc_mixed(span),
            [value, ..] => value,
        }
    }

    fn lower_generic_parameters(
        &mut self,
        parameters: Option<&'scratch GenericParameters<'scratch>>,
    ) -> &'arena [TypeAnnotation<'arena>] {
        match parameters {
            Some(parameters) => self.arena.alloc_slice_fill_iter(parameters.entries.iter().map(|entry| {
                TypeAnnotation { span: entry.inner.span(), kind: self.lower_type_annotation_kind(&entry.inner) }
            })),
            None => &[],
        }
    }

    fn lower_single_parameter(
        &mut self,
        parameter: &'scratch SingleGenericParameter<'scratch>,
    ) -> &'arena TypeAnnotation<'arena> {
        self.lower_type_annotation(&parameter.entry.inner)
    }

    fn class_like_string_parameter(
        &mut self,
        parameter: Option<&'scratch SingleGenericParameter<'scratch>>,
        span: Span,
    ) -> &'arena TypeAnnotation<'arena> {
        match parameter {
            Some(parameter) => self.lower_single_parameter(parameter),
            None => self.arena.alloc(TypeAnnotation { span, kind: TypeAnnotationKind::Wildcard }),
        }
    }

    fn alloc_mixed(&self, span: Span) -> &'arena TypeAnnotation<'arena> {
        self.arena.alloc(TypeAnnotation { span, kind: TypeAnnotationKind::Mixed(false) })
    }

    fn alloc_array_key(&self, span: Span) -> &'arena TypeAnnotation<'arena> {
        self.arena.alloc(TypeAnnotation { span, kind: self.array_key_kind() })
    }

    fn array_key_kind(&self) -> TypeAnnotationKind<'arena> {
        TypeAnnotationKind::ArrayKey
    }

    fn scalar_kind(&self) -> TypeAnnotationKind<'arena> {
        TypeAnnotationKind::Scalar
    }

    fn string(
        &self,
        span: Span,
        casing: Option<StringCasing>,
        literal: Option<StringLiteral<'arena>>,
        non_empty: bool,
        numeric: bool,
        truthy: bool,
        callable: bool,
    ) -> TypeAnnotationKind<'arena> {
        TypeAnnotationKind::String(StringTypeAnnotation { span, casing, literal, non_empty, numeric, truthy, callable })
    }

    fn lower_shape_fields(
        &mut self,
        span: Span,
        fields: &'scratch [ShapeField<'scratch>],
    ) -> Delimited<'arena, ShapeTypeAnnotationField<'arena>> {
        let items = self.arena.alloc_slice_fill_iter(fields.iter().enumerate().map(|(index, field)| {
            let (key, optional) = match &field.key {
                Some(field_key) => (self.lower_shape_key(&field_key.key), field_key.question_mark.is_some()),
                None => (ShapeTypeAnnotationKey::Integer(index as i64), false),
            };

            ShapeTypeAnnotationField {
                span: field.span(),
                key,
                optional,
                value: TypeAnnotation { span: field.value.span(), kind: self.lower_type_annotation_kind(field.value) },
            }
        }));

        Delimited { span, items }
    }

    fn lower_shape_key(&mut self, key: &ShapeKey<'scratch>) -> ShapeTypeAnnotationKey<'arena> {
        match key {
            ShapeKey::String { value, .. } => ShapeTypeAnnotationKey::String(self.interner.intern(value)),
            ShapeKey::Integer { value, .. } => ShapeTypeAnnotationKey::Integer(*value),
            ShapeKey::ClassLikeConstant { class_name, constant_name, .. } => ShapeTypeAnnotationKey::ClassLikeConstant(
                self.resolve_phpdoc_class(class_name),
                self.phpdoc_name(constant_name),
            ),
        }
    }

    fn lower_callable(&mut self, callable: &'scratch CallableType<'scratch>) -> TypeAnnotationKind<'arena> {
        let kind = match callable.kind {
            PHPDocCallableTypeKind::Callable => CallableTypeKind::Callable,
            PHPDocCallableTypeKind::PureCallable => CallableTypeKind::PureCallable,
            PHPDocCallableTypeKind::Closure => CallableTypeKind::Closure,
            PHPDocCallableTypeKind::PureClosure => CallableTypeKind::PureClosure,
        };

        let parameters = callable.specification.as_ref().map(|specification| Delimited {
            span: specification.parameters.left_parenthesis.join(specification.parameters.right_parenthesis),
            items: self.arena.alloc_slice_fill_iter(specification.parameters.entries.iter().map(|parameter| {
                CallableTypeAnnotationParameter {
                    span: parameter.span(),
                    r#type: parameter
                        .parameter_type
                        .as_ref()
                        .map(|parameter_type| self.lower_type_annotation(parameter_type)),
                    variadic: parameter.ellipsis.is_some(),
                    by_reference: parameter.ampersand.is_some(),
                    variable: parameter.variable.as_ref().map(|variable| self.phpdoc_variable(variable)),
                    has_default: parameter.equals.is_some(),
                }
            })),
        });

        let r#return = callable.specification.as_ref().and_then(|specification| {
            specification.return_type.as_ref().map(|return_type| self.lower_type_annotation(return_type.return_type))
        });

        TypeAnnotationKind::Callable(CallableTypeAnnotation { span: callable.span(), kind, parameters, r#return })
    }

    fn lower_member_reference_selector(
        &mut self,
        selector: &PHPDocMemberReferenceSelector<'scratch>,
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

    fn int_bound(&self, value: &IntOrKeyword<'scratch>) -> Option<i64> {
        match value {
            IntOrKeyword::Int(literal) => Some(literal.value as i64),
            IntOrKeyword::NegativeInt { int, .. } => Some(-(int.value as i64)),
            IntOrKeyword::Keyword(_) => None,
        }
    }

    pub(crate) fn phpdoc_name(&mut self, identifier: &PHPDocIdentifier<'scratch>) -> Name<'arena> {
        Name { span: identifier.span, value: self.interner.intern(identifier.value) }
    }

    pub(crate) fn phpdoc_variable(&mut self, variable: &PHPDocVariable<'scratch>) -> DirectVariable<'arena> {
        DirectVariable { span: variable.span, name: self.interner.intern(variable.value) }
    }

    pub(crate) fn lower_named_type(&mut self, ty: &'scratch Type<'scratch>) -> Option<NamedTypeAnnotation<'arena>> {
        match ty {
            Type::Parenthesized(parenthesized) => self.lower_named_type(parenthesized.inner),
            Type::Reference(reference) => {
                Some(self.named_from_reference_kind(reference.kind, reference.parameters.as_ref()))
            }
            _ => None,
        }
    }

    pub(crate) fn lower_named_types(&mut self, ty: &'scratch Type<'scratch>) -> &'arena [NamedTypeAnnotation<'arena>] {
        let mut members = Vec::new_in(self.arena);
        self.flatten_named_types(ty, &mut members);

        members.leak()
    }

    fn flatten_named_types(
        &mut self,
        ty: &'scratch Type<'scratch>,
        members: &mut Vec<'arena, NamedTypeAnnotation<'arena>, A>,
    ) {
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
