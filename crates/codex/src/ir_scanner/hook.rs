use mago_database::file::File;
use mago_hir::ir::expression::Access;
use mago_hir::ir::expression::Expression;
use mago_hir::ir::expression::ExpressionKind;
use mago_hir::ir::expression::selector::MemberSelector;
use mago_hir::ir::hook::Hook;
use mago_hir::ir::hook::HookBody;
use mago_hir::ir::member::HookedProperty;
use mago_hir::ir::member::PropertyFlags;
use mago_hir::ir::modifier::ModifierKind;
use mago_hir::ir::parameter::Parameter;
use mago_hir::ir::variable::Variable;
use mago_hir::walker::MutWalker;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::word;

use crate::metadata::constant::ConstantMetadata;

use crate::ir_scanner::attribute::scan_attributes;
use crate::ir_scanner::inference::infer;
use crate::ir_scanner::member::has;
use crate::ir_scanner::member::read_visibility;
use crate::ir_scanner::member::write_visibility;
use crate::ir_scanner::ttype::merge_type_preserving_nullability;
use crate::ir_scanner::ttype::type_metadata_from_annotation;
use crate::ir_scanner::ttype::type_metadata_from_type;
use crate::ir_scanner::version_constraint_from;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::property::PropertyMetadata;
use crate::metadata::property_hook::PropertyHookMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;

pub fn scan_hooked_properties(
    metadata: &mut ClassLikeMetadata,
    properties: &[HookedProperty<'_, (), (), ()>],
    origin: MetadataFlags,
    file: &File,
    constants: &WordMap<ConstantMetadata>,
) {
    let classname = metadata.name;

    for property in properties {
        let is_readonly = has(property.modifiers, ModifierKind::Readonly);
        let read = read_visibility(property.modifiers);
        let write = write_visibility(property.modifiers, read, is_readonly);
        let type_declaration = property.r#type.map(|hint| type_metadata_from_type(hint, Some(classname)));
        let type_annotation = property.type_annotation.map(|annotation| {
            merge_type_preserving_nullability(
                type_metadata_from_annotation(annotation, Some(classname)),
                type_declaration.as_ref(),
            )
        });

        let item = property.item;
        let name = word(item.variable.name);

        let mut flags = origin;
        if is_readonly {
            flags |= MetadataFlags::READONLY;
        }
        if has(property.modifiers, ModifierKind::Abstract) {
            flags |= MetadataFlags::ABSTRACT;
        }
        if has(property.modifiers, ModifierKind::Static) {
            flags |= MetadataFlags::STATIC;
        }
        if has(property.modifiers, ModifierKind::Final) || property.flags.is_set(PropertyFlags::Final) {
            flags |= MetadataFlags::FINAL;
        }
        if property.flags.is_set(PropertyFlags::Deprecated) {
            flags |= MetadataFlags::DEPRECATED;
        }
        if property.flags.is_set(PropertyFlags::Internal) {
            flags |= MetadataFlags::INTERNAL;
        }
        if property.flags.is_set(PropertyFlags::Experimental) {
            flags |= MetadataFlags::EXPERIMENTAL;
        }
        if property.flags.is_set(PropertyFlags::API) {
            flags |= MetadataFlags::API;
        }
        if item.default_value.is_some() {
            flags |= MetadataFlags::HAS_DEFAULT;
        }

        let mut property_metadata = PropertyMetadata::new(VariableIdentifier(name), flags);
        property_metadata.version_constraint = version_constraint_from(property.version_constraint);
        property_metadata.set_name_span(Some(item.variable.span));
        property_metadata.set_span(Some(property.span));
        property_metadata.set_visibility(read, write);
        if let Some(type_annotation) = &type_annotation {
            property_metadata.set_type_metadata(Some(type_annotation.clone()));
        }
        property_metadata.set_type_declaration_metadata(type_declaration);

        if let Some(default_value) = item.default_value {
            property_metadata.set_default_type_metadata(infer(default_value, Some(classname), file, constants).map(
                |union| {
                    let mut type_metadata = TypeMetadata::new(union, item.variable.span);
                    type_metadata.inferred = true;
                    type_metadata
                },
            ));
        }

        for hook in property.hooks {
            let hook_metadata = property_hook(hook, &property_metadata, classname);
            property_metadata.hooks.insert(hook_metadata.name, hook_metadata);
        }

        let property_name = name.as_bytes().strip_prefix(b"$").unwrap_or(name.as_bytes());
        property_metadata.set_is_virtual(!hooks_reference_backing_store(property.hooks, property_name));

        metadata.add_property(name, property_metadata);
    }
}

pub(super) fn property_hook(
    hook: &Hook<'_, (), (), ()>,
    property: &PropertyMetadata,
    classname: Word,
) -> PropertyHookMetadata {
    let name = word(hook.name.value);
    let is_set = hook.name.value == b"set";
    let is_abstract = hook.body.is_none();

    let mut flags = MetadataFlags::empty();
    if has(hook.modifiers, ModifierKind::Final) {
        flags |= MetadataFlags::FINAL;
    }

    let parameter = if is_set {
        Some(match hook.parameters.first() {
            Some(parameter) => hook_parameter(parameter, property, classname),
            None => implicit_value_parameter(property, hook.name.span),
        })
    } else {
        None
    };

    let return_type_metadata =
        hook.return_type_annotation.map(|annotation| type_metadata_from_annotation(annotation, Some(classname)));

    PropertyHookMetadata::new(name, hook.name.span)
        .with_flags(flags)
        .with_parameter(parameter)
        .with_returns_by_ref(hook.return_by_reference)
        .with_is_abstract(is_abstract)
        .with_attributes(scan_attributes(hook.attributes))
        .with_return_type_metadata(return_type_metadata)
        .with_has_docblock(hook.has_docblock)
        .with_issues(Vec::new())
}

fn hook_parameter(
    parameter: &Parameter<'_, (), (), ()>,
    property: &PropertyMetadata,
    classname: Word,
) -> FunctionLikeParameterMetadata {
    let mut flags = MetadataFlags::empty();
    if parameter.is_by_reference {
        flags |= MetadataFlags::BY_REFERENCE;
    }

    let mut metadata = FunctionLikeParameterMetadata::new(
        VariableIdentifier(word(parameter.variable.name)),
        parameter.variable.span,
        parameter.variable.span,
        flags,
    );

    if let Some(hint) = parameter.r#type {
        metadata.set_type_declaration_metadata(Some(type_metadata_from_type(hint, None)));
    } else if let Some(property_type) = &property.type_metadata {
        metadata.set_type_declaration_metadata(Some(property_type.clone()));
    }

    if let Some(annotation) = parameter.type_annotation {
        let docblock_type = type_metadata_from_annotation(annotation, Some(classname));
        let merged = merge_type_preserving_nullability(docblock_type, metadata.type_declaration_metadata.as_ref());
        metadata.set_type_metadata(Some(merged));
    }

    metadata
}

fn implicit_value_parameter(property: &PropertyMetadata, span: mago_span::Span) -> FunctionLikeParameterMetadata {
    let mut metadata =
        FunctionLikeParameterMetadata::new(VariableIdentifier(word(b"$value")), span, span, MetadataFlags::empty());
    if let Some(property_type) = &property.type_metadata {
        metadata.set_type_declaration_metadata(Some(property_type.clone()));
    }

    metadata
}

struct BackingFinder<'name> {
    property_name: &'name [u8],
    found: bool,
    assignment_seen: bool,
}

impl<'arena> MutWalker<'arena, (), (), (), ()> for BackingFinder<'_> {
    fn walk_in_expression(&mut self, expression: &Expression<'arena, (), (), ()>, _context: &mut ()) {
        match &expression.kind {
            ExpressionKind::Assignment(_) => self.assignment_seen = true,
            ExpressionKind::Access(access) => {
                let (object, MemberSelector::Name(selector)) = (match access {
                    Access::Property(object, selector) | Access::NullsafeProperty(object, selector) => {
                        (object, selector)
                    }
                    _ => return,
                }) else {
                    return;
                };

                if let ExpressionKind::Variable(Variable::Direct(variable)) = &object.kind {
                    if variable.name == b"$this" && selector.value == self.property_name {
                        self.found = true;
                    }
                }
            }
            _ => {}
        }
    }
}

pub(super) fn hooks_reference_backing_store(hooks: &[Hook<'_, (), (), ()>], property_name: &[u8]) -> bool {
    for hook in hooks {
        let Some(body) = &hook.body else {
            continue;
        };

        let mut finder = BackingFinder { property_name, found: false, assignment_seen: false };
        match body {
            HookBody::Expression(expression) => finder.walk_expression(expression, &mut ()),
            HookBody::Statements(statements) => {
                for statement in *statements {
                    finder.walk_statement(statement, &mut ());
                }
            }
        }

        if finder.found {
            return true;
        }

        if hook.name.value == b"set" && matches!(body, HookBody::Expression(_)) && !finder.assignment_seen {
            return true;
        }
    }

    false
}
