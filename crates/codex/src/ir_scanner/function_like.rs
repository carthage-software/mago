use std::collections::BTreeMap;

use mago_database::file::File;
use mago_hir::ir::effect::annotation::AssertAnnotation;
use mago_hir::ir::effect::annotation::AssertAnnotationTarget;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::definition::AnonymousClass;
use mago_hir::ir::expression::definition::ArrowFunction;
use mago_hir::ir::expression::definition::Closure;
use mago_hir::ir::member::Method;
use mago_hir::ir::member::MethodFlags;
use mago_hir::ir::member::annotation::MethodAnnotation;
use mago_hir::ir::modifier::ModifierKind;
use mago_hir::ir::modifier::VisibilityKind;
use mago_hir::ir::parameter::Parameter;
use mago_hir::ir::statement::GlobalItem;
use mago_hir::ir::statement::Statement;
use mago_hir::ir::statement::definition::Function;
use mago_hir::ir::statement::definition::FunctionFlags;
use mago_hir::ir::r#type::annotation::ReferenceKind;
use mago_hir::ir::r#type::annotation::TypeAnnotationKind;
use mago_hir::ir::variable::Variable;
use mago_hir::walker::MutWalker;
use mago_span::Span;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;
use mago_word::concat_word;
use mago_word::word;

use crate::assertion::Assertion;
use crate::ir_scanner::attribute::scan_attributes;
use crate::ir_scanner::hook::hooks_reference_backing_store;
use crate::ir_scanner::hook::property_hook;
use crate::ir_scanner::inference::infer;
use crate::ir_scanner::member::has;
use crate::ir_scanner::member::read_visibility;
use crate::ir_scanner::member::write_visibility;
use crate::ir_scanner::ttype::merge_type_preserving_nullability;
use crate::ir_scanner::ttype::type_metadata_from_annotation;
use crate::ir_scanner::ttype::type_metadata_from_type;
use crate::ir_scanner::ttype::union_from_annotation;
use crate::ir_scanner::version_constraint_from;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::function_like::FunctionLikeKind;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::GenericParent;
use crate::misc::VariableIdentifier;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::get_list;
use crate::ttype::get_named_object;
use crate::ttype::get_non_empty_list;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

#[derive(Default)]
struct EffectFinder {
    has_throw: bool,
    has_yield: bool,
}

impl<'arena> MutWalker<'arena, (), (), (), ()> for EffectFinder {
    fn walk_in_expression(&mut self, expression: &Expression<'arena, (), (), ()>, _context: &mut ()) {
        match &expression.kind {
            ExpressionKind::Throw(_) => self.has_throw = true,
            ExpressionKind::Yield(_) => self.has_yield = true,
            _ => {}
        }
    }

    fn walk_closure(&mut self, _closure: &Closure<'arena, (), (), ()>, _context: &mut ()) {}
    fn walk_arrow_function(&mut self, _arrow: &ArrowFunction<'arena, (), (), ()>, _context: &mut ()) {}
    fn walk_anonymous_class(&mut self, _anonymous_class: &AnonymousClass<'arena, (), (), ()>, _context: &mut ()) {}
    fn walk_function(&mut self, _function: &Function<'arena, (), (), ()>, _context: &mut ()) {}
}

fn body_effects(statement: &Statement<'_, (), (), ()>) -> (bool, bool) {
    let mut finder = EffectFinder::default();
    finder.walk_statement(statement, &mut ());
    (finder.has_throw, finder.has_yield)
}

#[derive(Default)]
struct GlobalsFinder {
    globals: WordSet,
}

impl<'arena> MutWalker<'arena, (), (), (), ()> for GlobalsFinder {
    fn walk_in_global_item(&mut self, global_item: &GlobalItem<'arena, (), (), ()>, _context: &mut ()) {
        if let Variable::Direct(direct) = &global_item.variable {
            self.globals.insert(word(direct.name));
        }
    }

    fn walk_closure(&mut self, _closure: &Closure<'arena, (), (), ()>, _context: &mut ()) {}
    fn walk_arrow_function(&mut self, _arrow: &ArrowFunction<'arena, (), (), ()>, _context: &mut ()) {}
    fn walk_anonymous_class(&mut self, _anonymous_class: &AnonymousClass<'arena, (), (), ()>, _context: &mut ()) {}
    fn walk_function(&mut self, _function: &Function<'arena, (), (), ()>, _context: &mut ()) {}
}

fn body_globals(statement: &Statement<'_, (), (), ()>) -> WordSet {
    let mut finder = GlobalsFinder::default();
    finder.walk_statement(statement, &mut ());
    finder.globals
}

fn expression_effects(expression: &Expression<'_, (), (), ()>) -> (bool, bool) {
    let mut finder = EffectFinder::default();
    finder.walk_expression(expression, &mut ());
    (finder.has_throw, finder.has_yield)
}

fn apply_effect_flags(flags: &mut MetadataFlags, has_throw: bool, has_yield: bool) {
    if has_yield {
        *flags |= MetadataFlags::HAS_YIELD;
    }
    if has_throw {
        *flags |= MetadataFlags::HAS_THROW;
    }
}

pub(super) fn is_promoted(parameter: &Parameter<'_, (), (), ()>) -> bool {
    parameter.modifiers.iter().any(|modifier| {
        matches!(
            modifier.kind,
            ModifierKind::Public
                | ModifierKind::Protected
                | ModifierKind::Private
                | ModifierKind::PublicSet
                | ModifierKind::ProtectedSet
                | ModifierKind::PrivateSet
                | ModifierKind::Readonly
        )
    })
}

fn scan_parameters(
    parameters: &[Parameter<'_, (), (), ()>],
    classname: Option<Word>,
    origin: MetadataFlags,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> Vec<FunctionLikeParameterMetadata> {
    parameters
        .iter()
        .map(|parameter| {
            let name = word(parameter.variable.name);

            let mut flags = origin;
            if parameter.is_variadic {
                flags |= MetadataFlags::VARIADIC;
            }
            if parameter.is_by_reference {
                flags |= MetadataFlags::BY_REFERENCE;
            }
            if parameter.default_value.is_some() {
                flags |= MetadataFlags::HAS_DEFAULT;
            }
            if is_promoted(parameter) {
                flags |= MetadataFlags::PROMOTED_PROPERTY;
            }

            let mut metadata = FunctionLikeParameterMetadata::new(
                VariableIdentifier(name),
                parameter.variable.span,
                parameter.variable.span,
                flags,
            );
            metadata.set_attributes(scan_attributes(parameter.attributes));

            let declaration = parameter.r#type.map(|hint| type_metadata_from_type(hint, classname));
            if let Some(annotation) = parameter.type_annotation {
                let docblock = type_metadata_from_annotation(annotation, classname);
                metadata.set_type_metadata(Some(merge_type_preserving_nullability(docblock, declaration.as_ref())));
            }
            metadata.set_type_declaration_metadata(declaration);

            if let Some(out_annotation) = parameter.out_annotation {
                metadata.out_type = Some(type_metadata_from_annotation(out_annotation, classname));
            }

            if let Some(default_value) = parameter.default_value {
                metadata.default_type = infer(default_value, classname, file, constants).map(|union| {
                    let mut type_metadata = TypeMetadata::new(union, parameter.variable.span);
                    type_metadata.inferred = true;
                    type_metadata
                });
            }

            metadata
        })
        .collect()
}

fn apply_asserts(map: &mut BTreeMap<Word, Vec<Assertion>>, asserts: &[AssertAnnotation<'_>], classname: Option<Word>) {
    for assert in asserts {
        let name = match assert.target {
            AssertAnnotationTarget::Variable(variable) => word(variable.name),
            AssertAnnotationTarget::Property(variable, selector) => {
                concat_word!(variable.name, b"->", selector.value)
            }
            AssertAnnotationTarget::Method(variable, selector) => {
                concat_word!(variable.name, b"->", selector.value, b"()")
            }
        };

        if !assert.equality
            && let Some(assertion) = assertion_predicate(&assert.r#type.kind, assert.negated)
        {
            map.entry(name).or_default().push(assertion);
            continue;
        }

        for atomic in union_from_annotation(&assert.r#type.kind, classname).types.into_owned() {
            let assertion = match (assert.equality, assert.negated) {
                (true, true) => Assertion::IsNotIdentical(atomic),
                (true, false) => Assertion::IsIdentical(atomic),
                (false, true) => Assertion::IsNotType(atomic),
                (false, false) => Assertion::IsType(atomic),
            };

            map.entry(name).or_default().push(assertion);
        }
    }
}

/// Recognizes the special `@assert` predicate keywords (`empty`, `non-empty`,
/// `truthy`, `falsy`) that are written in type position but denote a
/// truthiness/emptiness assertion rather than a type. Negation flips each
/// keyword to its counterpart, mirroring the CST scanner.
fn assertion_predicate(kind: &TypeAnnotationKind<'_>, negated: bool) -> Option<Assertion> {
    let TypeAnnotationKind::Named(named) = kind else {
        return None;
    };

    if !named.type_arguments.is_empty() {
        return None;
    }

    let ReferenceKind::Identifier(identifier) = named.kind else {
        return None;
    };

    let value = identifier.value;
    if value.eq_ignore_ascii_case(b"truthy") {
        Some(if negated { Assertion::Falsy } else { Assertion::Truthy })
    } else if value.eq_ignore_ascii_case(b"falsy") {
        Some(if negated { Assertion::Truthy } else { Assertion::Falsy })
    } else if value.eq_ignore_ascii_case(b"empty") {
        Some(if negated { Assertion::NonEmpty } else { Assertion::Empty })
    } else if value.eq_ignore_ascii_case(b"non-empty") {
        Some(if negated { Assertion::Empty } else { Assertion::NonEmpty })
    } else {
        None
    }
}

#[must_use]
pub fn promoted_property(
    parameter: &Parameter<'_, (), (), ()>,
    parameter_metadata: &FunctionLikeParameterMetadata,
    origin: MetadataFlags,
    classname: Word,
) -> PropertyMetadata {
    let mut flags = MetadataFlags::PROMOTED_PROPERTY | origin;
    if parameter_metadata.flags.has_default() {
        flags |= MetadataFlags::HAS_DEFAULT;
    }
    let is_readonly = has(parameter.modifiers, ModifierKind::Readonly);
    if is_readonly {
        flags |= MetadataFlags::READONLY;
    }
    if has(parameter.modifiers, ModifierKind::Abstract) {
        flags |= MetadataFlags::ABSTRACT;
    }
    if has(parameter.modifiers, ModifierKind::Static) {
        flags |= MetadataFlags::STATIC;
    }

    let read = read_visibility(parameter.modifiers);
    let write = write_visibility(parameter.modifiers, read, is_readonly);

    let mut property = PropertyMetadata::new(*parameter_metadata.get_name(), flags);
    property.set_default_type_metadata(parameter_metadata.get_default_type().cloned());
    property.set_name_span(Some(parameter_metadata.get_name_span()));
    property.set_span(Some(parameter.variable.span));
    property.set_visibility(read, write);
    property.set_type_declaration_metadata(parameter_metadata.get_type_declaration_metadata().cloned());

    if let Some(type_metadata) = parameter_metadata.type_metadata.as_ref() {
        if type_metadata.from_docblock {
            property.type_metadata = Some(type_metadata.clone());
        }
    }

    if !parameter.hooks.is_empty() {
        for hook in parameter.hooks {
            let hook_metadata = property_hook(hook, &property, classname);
            property.hooks.insert(hook_metadata.name, hook_metadata);
        }

        let property_name = parameter.variable.name.strip_prefix(b"$").unwrap_or(parameter.variable.name);
        property.set_is_virtual(!hooks_reference_backing_store(parameter.hooks, property_name));
    }

    property
}

#[must_use]
pub fn scan_method(
    method: &Method<'_, (), (), ()>,
    class_name: Word,
    origin: MetadataFlags,
    class_context: &TypeResolutionContext,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> FunctionLikeMetadata {
    let lookup_name = mago_word::ascii_lowercase_word(method.name.value);
    let display_name = word(method.name.value);

    let mut flags = origin;
    if method.return_by_reference {
        flags |= MetadataFlags::BY_REFERENCE;
    }
    apply_method_flags(&mut flags, method);

    let mut metadata =
        FunctionLikeMetadata::new(FunctionLikeKind::Method, lookup_name, display_name, method.span, flags);
    metadata.name_span = Some(method.name.span);
    metadata.attributes = scan_attributes(method.attributes);
    metadata.version_constraint = version_constraint_from(method.version_constraint);
    let (has_throw, has_yield) = method.body.map_or((false, false), body_effects);
    metadata.globals_accessed = method.body.map(body_globals).unwrap_or_default();
    apply_effect_flags(&mut metadata.flags, has_throw, has_yield);

    let is_constructor = method.name.value.eq_ignore_ascii_case(b"__construct");
    if let Some(method_metadata) = metadata.get_method_metadata_mut() {
        method_metadata.is_final =
            has(method.modifiers, ModifierKind::Final) || method.flags.is_set(MethodFlags::Final);
        method_metadata.is_abstract = has(method.modifiers, ModifierKind::Abstract) || method.body.is_none();
        method_metadata.is_static = has(method.modifiers, ModifierKind::Static);
        method_metadata.is_constructor = is_constructor;
        method_metadata.visibility = read_visibility(method.modifiers);
    }

    metadata.set_parameters(scan_parameters(method.parameters, Some(class_name), origin, file, constants));

    let return_declaration = method.return_type.map(|hint| type_metadata_from_type(hint, Some(class_name)));
    if let Some(annotation) = method.return_type_annotation {
        let docblock = type_metadata_from_annotation(annotation, Some(class_name));
        metadata
            .set_return_type_metadata(Some(merge_type_preserving_nullability(docblock, return_declaration.as_ref())));
    }
    metadata.set_return_type_declaration_metadata(return_declaration);

    apply_ignore_return_union_flags(&mut metadata);

    metadata.thrown_types =
        method.throws.iter().map(|throws| type_metadata_from_annotation(throws.r#type, Some(class_name))).collect();

    apply_asserts(&mut metadata.assertions, method.asserts, Some(class_name));
    apply_asserts(&mut metadata.if_true_assertions, method.asserts_if_true, Some(class_name));
    apply_asserts(&mut metadata.if_false_assertions, method.asserts_if_false, Some(class_name));

    metadata.has_docblock = method.flags.is_set(MethodFlags::HasDocblock);
    metadata.assertions_inferred = method.flags.is_set(MethodFlags::AssertionsInferred);

    let mut context = class_context.clone();

    crate::ir_scanner::generics::apply_function_like_templates(
        &mut metadata,
        &mut context,
        method.type_parameter_annotations,
        GenericParent::FunctionLike((class_name, lookup_name)),
        Some(class_name),
    );

    if method.flags.is_set(MethodFlags::HasDocblock) || !context.is_empty() {
        metadata.type_resolution_context = Some(context);
    }

    metadata
}

#[must_use]
pub fn scan_function(
    function: &Function<'_, (), (), ()>,
    origin: MetadataFlags,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> FunctionLikeMetadata {
    let lookup_name = mago_word::ascii_lowercase_word(function.name.value);
    let display_name = word(function.name.value);

    let mut flags = origin;
    if function.return_by_reference {
        flags |= MetadataFlags::BY_REFERENCE;
    }
    apply_function_flags(&mut flags, function);

    let mut metadata =
        FunctionLikeMetadata::new(FunctionLikeKind::Function, lookup_name, display_name, function.name.span, flags);
    metadata.name_span = Some(function.name.span);
    metadata.attributes = scan_attributes(function.attributes);
    metadata.version_constraint = version_constraint_from(function.version_constraint);
    let (has_throw, has_yield) = body_effects(function.body);
    metadata.globals_accessed = body_globals(function.body);
    apply_effect_flags(&mut metadata.flags, has_throw, has_yield);

    metadata.set_parameters(scan_parameters(function.parameters, None, origin, file, constants));

    let return_declaration = function.return_type.map(|hint| type_metadata_from_type(hint, None));
    if let Some(annotation) = function.return_type_annotation {
        let docblock = type_metadata_from_annotation(annotation, None);
        metadata
            .set_return_type_metadata(Some(merge_type_preserving_nullability(docblock, return_declaration.as_ref())));
    }
    metadata.set_return_type_declaration_metadata(return_declaration);

    apply_ignore_return_union_flags(&mut metadata);

    metadata.thrown_types =
        function.throws_annotations.iter().map(|throws| type_metadata_from_annotation(throws.r#type, None)).collect();

    apply_asserts(&mut metadata.assertions, function.assert_annotations, None);
    apply_asserts(&mut metadata.if_true_assertions, function.assert_if_true_annotations, None);
    apply_asserts(&mut metadata.if_false_assertions, function.assert_if_false_annotations, None);

    metadata.has_docblock = function.flags.is_set(FunctionFlags::HasDocblock);
    metadata.assertions_inferred = function.flags.is_set(FunctionFlags::AssertionsInferred);

    let mut context = TypeResolutionContext::new();
    crate::ir_scanner::generics::apply_function_like_templates(
        &mut metadata,
        &mut context,
        function.type_parameter_annotations,
        GenericParent::FunctionLike((mago_word::empty_word(), lookup_name)),
        None,
    );

    if function.flags.is_set(FunctionFlags::HasDocblock) || !context.is_empty() {
        metadata.type_resolution_context = Some(context);
    }

    metadata
}

fn synthetic_enum_method(name: &'static [u8], span: Span) -> FunctionLikeMetadata {
    let mut metadata =
        FunctionLikeMetadata::new(FunctionLikeKind::Method, word(name), word(name), span, MetadataFlags::POPULATED);
    metadata.name_span = Some(span);
    if let Some(method_metadata) = metadata.get_method_metadata_mut() {
        method_metadata.is_final = true;
        method_metadata.is_static = true;
    }

    metadata
}

#[must_use]
pub fn enum_from_method(enum_name: Word, span: Span, backing: TAtomic) -> FunctionLikeMetadata {
    let mut metadata = synthetic_enum_method(b"from", span);

    let mut value =
        FunctionLikeParameterMetadata::new(VariableIdentifier(word("$value")), span, span, MetadataFlags::empty());
    value.set_type_declaration_metadata(Some(TypeMetadata::new(TUnion::from_vec(vec![backing]), span)));
    metadata.set_parameters([value]);

    metadata.set_return_type_declaration_metadata(Some(TypeMetadata::new(
        TUnion::from_vec(vec![TAtomic::Object(TObject::new_enum(enum_name))]),
        span,
    )));
    metadata.thrown_types = vec![TypeMetadata::new(get_named_object(word("ValueError"), None), span)];

    metadata
}

#[must_use]
pub fn enum_try_from_method(enum_name: Word, span: Span, backing: TAtomic) -> FunctionLikeMetadata {
    let mut metadata = synthetic_enum_method(b"tryFrom", span);

    let mut value =
        FunctionLikeParameterMetadata::new(VariableIdentifier(word("$value")), span, span, MetadataFlags::empty());
    value.set_type_declaration_metadata(Some(TypeMetadata::new(TUnion::from_vec(vec![backing]), span)));
    metadata.set_parameters([value]);

    metadata.set_return_type_declaration_metadata(Some(TypeMetadata::new(
        TUnion::from_vec(vec![TAtomic::Object(TObject::new_enum(enum_name)), TAtomic::Null]),
        span,
    )));

    metadata
}

#[must_use]
pub fn enum_cases_method(enum_name: Word, span: Span, has_cases: bool) -> FunctionLikeMetadata {
    let mut metadata = synthetic_enum_method(b"cases", span);

    let element = TUnion::from_vec(vec![TAtomic::Object(TObject::new_enum(enum_name))]);
    let return_type = if has_cases { get_non_empty_list(element) } else { get_list(element) };
    metadata.set_return_type_declaration_metadata(Some(TypeMetadata::new(return_type, span)));

    metadata
}

#[must_use]
pub fn scan_closure(
    closure: &Closure<'_, (), (), ()>,
    name: Word,
    span: Span,
    origin: MetadataFlags,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> FunctionLikeMetadata {
    let mut flags = origin;
    if closure.return_by_reference {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::Closure, name, name, span, flags);
    metadata.attributes = scan_attributes(closure.attributes);
    metadata.version_constraint = version_constraint_from(closure.version_constraint);
    metadata.has_docblock = closure.has_docblock;
    let (has_throw, has_yield) = body_effects(closure.body);
    metadata.globals_accessed = body_globals(closure.body);
    apply_effect_flags(&mut metadata.flags, has_throw, has_yield);
    metadata.set_parameters(scan_parameters(closure.parameters, None, origin, file, constants));

    let return_declaration = closure.return_type.map(|hint| type_metadata_from_type(hint, None));
    if let Some(annotation) = closure.return_type_annotation {
        let docblock = type_metadata_from_annotation(annotation, None);
        metadata
            .set_return_type_metadata(Some(merge_type_preserving_nullability(docblock, return_declaration.as_ref())));
    }
    metadata.set_return_type_declaration_metadata(return_declaration);

    apply_ignore_return_union_flags(&mut metadata);

    metadata.thrown_types =
        closure.throws_annotations.iter().map(|throws| type_metadata_from_annotation(throws.r#type, None)).collect();
    apply_asserts(&mut metadata.assertions, closure.assert_annotations, None);
    apply_asserts(&mut metadata.if_true_assertions, closure.assert_if_true_annotations, None);
    apply_asserts(&mut metadata.if_false_assertions, closure.assert_if_false_annotations, None);
    metadata.assertions_inferred = closure.assertions_inferred;

    let mut context = TypeResolutionContext::new();
    crate::ir_scanner::generics::apply_inherited_templates(&mut context, closure.inherited_type_parameters);
    crate::ir_scanner::generics::apply_function_like_templates(
        &mut metadata,
        &mut context,
        closure.type_parameter_annotations,
        GenericParent::FunctionLike((mago_word::empty_word(), name)),
        None,
    );
    if !context.is_empty() {
        metadata.type_resolution_context = Some(context);
    }

    metadata
}

#[must_use]
pub fn scan_arrow_function(
    arrow: &ArrowFunction<'_, (), (), ()>,
    name: Word,
    span: Span,
    origin: MetadataFlags,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) -> FunctionLikeMetadata {
    let mut flags = origin;
    if arrow.return_by_reference {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let mut metadata = FunctionLikeMetadata::new(FunctionLikeKind::ArrowFunction, name, name, span, flags);
    metadata.attributes = scan_attributes(arrow.attributes);
    metadata.version_constraint = version_constraint_from(arrow.version_constraint);
    metadata.has_docblock = arrow.has_docblock;
    let (has_throw, has_yield) = expression_effects(arrow.expression);
    apply_effect_flags(&mut metadata.flags, has_throw, has_yield);
    metadata.set_parameters(scan_parameters(arrow.parameters, None, origin, file, constants));

    let return_declaration = arrow.return_type.map(|hint| type_metadata_from_type(hint, None));
    if let Some(annotation) = arrow.return_type_annotation {
        let docblock = type_metadata_from_annotation(annotation, None);
        metadata
            .set_return_type_metadata(Some(merge_type_preserving_nullability(docblock, return_declaration.as_ref())));
    }
    metadata.set_return_type_declaration_metadata(return_declaration);

    apply_ignore_return_union_flags(&mut metadata);

    metadata.thrown_types =
        arrow.throws_annotations.iter().map(|throws| type_metadata_from_annotation(throws.r#type, None)).collect();
    apply_asserts(&mut metadata.assertions, arrow.assert_annotations, None);
    apply_asserts(&mut metadata.if_true_assertions, arrow.assert_if_true_annotations, None);
    apply_asserts(&mut metadata.if_false_assertions, arrow.assert_if_false_annotations, None);
    metadata.assertions_inferred = arrow.assertions_inferred;

    let mut context = TypeResolutionContext::new();
    crate::ir_scanner::generics::apply_inherited_templates(&mut context, arrow.inherited_type_parameters);
    crate::ir_scanner::generics::apply_function_like_templates(
        &mut metadata,
        &mut context,
        arrow.type_parameter_annotations,
        GenericParent::FunctionLike((mago_word::empty_word(), name)),
        None,
    );
    if !context.is_empty() {
        metadata.type_resolution_context = Some(context);
    }

    metadata
}

#[must_use]
pub fn magic_method(annotation: &MethodAnnotation<'_, (), (), ()>, class_name: Word) -> FunctionLikeMetadata {
    let lookup_name = mago_word::ascii_lowercase_word(annotation.name.value);

    let mut flags = MetadataFlags::MAGIC_METHOD;
    if annotation.r#static {
        flags |= MetadataFlags::STATIC;
    }

    let mut metadata = FunctionLikeMetadata::new(
        FunctionLikeKind::Method,
        lookup_name,
        word(annotation.name.value),
        annotation.span,
        flags,
    );
    if let Some(method_metadata) = metadata.get_method_metadata_mut() {
        method_metadata.is_static = annotation.r#static;
        if let Some(visibility) = annotation.visibility {
            method_metadata.visibility = match visibility.kind {
                VisibilityKind::Public => Visibility::Public,
                VisibilityKind::Protected => Visibility::Protected,
                VisibilityKind::Private => Visibility::Private,
            };
        }
    }

    let parameters: Vec<FunctionLikeParameterMetadata> = annotation
        .parameters
        .iter()
        .map(|parameter| {
            let mut flags = MetadataFlags::empty();
            if parameter.is_variadic {
                flags |= MetadataFlags::VARIADIC;
            }
            if parameter.is_by_reference {
                flags |= MetadataFlags::BY_REFERENCE;
            }
            if parameter.default_value.is_some() {
                flags |= MetadataFlags::HAS_DEFAULT;
            }

            let mut parameter_metadata = FunctionLikeParameterMetadata::new(
                VariableIdentifier(word(parameter.variable.name)),
                parameter.variable.span,
                parameter.variable.span,
                flags,
            );
            parameter_metadata.set_type_declaration_metadata(
                parameter.r#type.map(|annotation| type_metadata_from_annotation(annotation, Some(class_name))),
            );
            parameter_metadata
        })
        .collect();
    metadata.set_parameters(parameters);

    metadata.return_type_metadata =
        annotation.return_type.map(|annotation| type_metadata_from_annotation(annotation, Some(class_name)));

    metadata
}

fn apply_method_flags(flags: &mut MetadataFlags, method: &Method<'_, (), (), ()>) {
    let marker = method.flags;
    if marker.is_set(MethodFlags::Deprecated) {
        *flags |= MetadataFlags::DEPRECATED;
    }
    if marker.is_set(MethodFlags::Internal) {
        *flags |= MetadataFlags::INTERNAL;
    }
    if marker.is_set(MethodFlags::Experimental) {
        *flags |= MetadataFlags::EXPERIMENTAL;
    }
    if marker.is_set(MethodFlags::API) {
        *flags |= MetadataFlags::API;
    }
    if marker.is_set(MethodFlags::Pure) {
        *flags |= MetadataFlags::PURE;
    }
    if marker.is_set(MethodFlags::MutationFree) {
        *flags |= MetadataFlags::MUTATION_FREE;
        *flags |= MetadataFlags::EXTERNAL_MUTATION_FREE;
    }
    if marker.is_set(MethodFlags::ExternalMutationFree) {
        *flags |= MetadataFlags::EXTERNAL_MUTATION_FREE;
    }
    if marker.is_set(MethodFlags::SuspendsFiber) {
        *flags |= MetadataFlags::SUSPENDS_FIBER;
    }
    if marker.is_set(MethodFlags::IgnoreNullableReturnType) {
        *flags |= MetadataFlags::IGNORE_NULLABLE_RETURN;
    }
    if marker.is_set(MethodFlags::IgnoreFalsableReturnType) {
        *flags |= MetadataFlags::IGNORE_FALSABLE_RETURN;
    }
    if marker.is_set(MethodFlags::InheritDoc) {
        *flags |= MetadataFlags::INHERITS_DOCS;
    }
    if marker.is_set(MethodFlags::NoNamedArguments) {
        *flags |= MetadataFlags::NO_NAMED_ARGUMENTS;
    }
    if marker.is_set(MethodFlags::MustUse) {
        *flags |= MetadataFlags::MUST_USE;
    }
}

fn apply_function_flags(flags: &mut MetadataFlags, function: &Function<'_, (), (), ()>) {
    let marker = function.flags;
    if marker.is_set(FunctionFlags::Deprecated) {
        *flags |= MetadataFlags::DEPRECATED;
    }
    if marker.is_set(FunctionFlags::Internal) {
        *flags |= MetadataFlags::INTERNAL;
    }
    if marker.is_set(FunctionFlags::Experimental) {
        *flags |= MetadataFlags::EXPERIMENTAL;
    }
    if marker.is_set(FunctionFlags::API) {
        *flags |= MetadataFlags::API;
    }
    if marker.is_set(FunctionFlags::Pure) {
        *flags |= MetadataFlags::PURE;
    }
    if marker.is_set(FunctionFlags::MutationFree) {
        *flags |= MetadataFlags::MUTATION_FREE;
        *flags |= MetadataFlags::EXTERNAL_MUTATION_FREE;
    }
    if marker.is_set(FunctionFlags::ExternalMutationFree) {
        *flags |= MetadataFlags::EXTERNAL_MUTATION_FREE;
    }
    if marker.is_set(FunctionFlags::SuspendsFiber) {
        *flags |= MetadataFlags::SUSPENDS_FIBER;
    }
    if marker.is_set(FunctionFlags::IgnoreNullableReturnType) {
        *flags |= MetadataFlags::IGNORE_NULLABLE_RETURN;
    }
    if marker.is_set(FunctionFlags::IgnoreFalsableReturnType) {
        *flags |= MetadataFlags::IGNORE_FALSABLE_RETURN;
    }
    if marker.is_set(FunctionFlags::NoNamedArguments) {
        *flags |= MetadataFlags::NO_NAMED_ARGUMENTS;
    }
    if marker.is_set(FunctionFlags::MustUse) {
        *flags |= MetadataFlags::MUST_USE;
    }
}

fn apply_ignore_return_union_flags(metadata: &mut FunctionLikeMetadata) {
    let ignore_nullable = metadata.flags.ignore_nullable_return();
    let ignore_falsable = metadata.flags.ignore_falsable_return();
    if !ignore_nullable && !ignore_falsable {
        return;
    }

    if let Some(return_type) = metadata.return_type_metadata.as_mut() {
        return_type.type_union.set_ignore_nullable_issues(ignore_nullable);
        return_type.type_union.set_ignore_falsable_issues(ignore_falsable);
    }
    if let Some(return_type) = metadata.return_type_declaration_metadata.as_mut() {
        return_type.type_union.set_ignore_nullable_issues(ignore_nullable);
        return_type.type_union.set_ignore_falsable_issues(ignore_falsable);
    }
}
