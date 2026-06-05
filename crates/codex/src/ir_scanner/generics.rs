use mago_hir::ir::generics::Variance as IrVariance;
use mago_hir::ir::generics::annotation::InheritedTemplateAnnotation;
use mago_hir::ir::generics::annotation::TypeParameterAnnotation;
use mago_hir::ir::inheritance::annotation::ExtendsAnnotation;
use mago_hir::ir::inheritance::annotation::ImplementsAnnotation;
use mago_hir::ir::inheritance::annotation::MixinAnnotation;
use mago_hir::ir::member::TraitUse;
use mago_hir::ir::member::annotation::ImportedTypeAliasAnnotation;
use mago_hir::ir::member::annotation::TypeAliasAnnotation;
use mago_word::Word;
use mago_word::ascii_lowercase_word;
use mago_word::word;

use crate::ir_scanner::ttype::generic_parent;
use crate::ir_scanner::ttype::type_metadata_from_annotation;
use crate::ir_scanner::ttype::union_from_annotation;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::function_like::FunctionLikeMetadata;
use crate::misc::GenericParent;
use crate::ttype::get_mixed;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::template::GenericTemplate;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;

#[must_use]
pub fn class_template_context(metadata: &ClassLikeMetadata) -> TypeResolutionContext {
    let mut context = TypeResolutionContext::new();
    for (name, template) in &metadata.template_types {
        context.get_template_definitions_mut().insert(*name, vec![template.clone()]);
    }

    context = context.with_type_aliases(metadata.type_aliases.keys().copied().collect());
    for (local, (source, original, _span)) in &metadata.imported_type_aliases {
        context = context.with_imported_type_alias(*local, *source, *original);
    }

    context
}

pub fn apply_function_like_templates(
    metadata: &mut FunctionLikeMetadata,
    context: &mut TypeResolutionContext,
    annotations: &[TypeParameterAnnotation<'_>],
    defining_entity: GenericParent,
    classname: Option<Word>,
) {
    for annotation in annotations {
        let name = word(annotation.name.value);
        let constraint = annotation.bound.map_or_else(get_mixed, |bound| union_from_annotation(&bound.kind, classname));
        let default = annotation.default.map(|default| union_from_annotation(&default.kind, classname));

        let template = GenericTemplate::new(defining_entity, constraint).with_default(default);
        metadata.add_template_type(name, template.clone());
        context.get_template_definitions_mut().insert(name, vec![template]);
    }
}

pub fn apply_inherited_templates(context: &mut TypeResolutionContext, inherited: &[InheritedTemplateAnnotation<'_>]) {
    for annotation in inherited {
        let name = word(annotation.name.value);
        let constraint = annotation.bound.map_or_else(get_mixed, |bound| union_from_annotation(&bound.kind, None));
        let default = annotation.default.map(|default| union_from_annotation(&default.kind, None));
        let template =
            GenericTemplate::new(generic_parent(&annotation.defining_entity), constraint).with_default(default);
        context.get_template_definitions_mut().insert(name, vec![template]);
    }
}

fn map_variance(variance: IrVariance) -> Variance {
    match variance {
        IrVariance::Invariant => Variance::Invariant,
        IrVariance::Covariant => Variance::Covariant,
        IrVariance::Contravariant => Variance::Contravariant,
    }
}

pub fn scan_template_types(metadata: &mut ClassLikeMetadata, annotations: &[TypeParameterAnnotation<'_>]) {
    if annotations.is_empty() {
        return;
    }

    let classname = metadata.name;
    let mut variances = Vec::with_capacity(annotations.len());
    for annotation in annotations {
        let name = word(annotation.name.value);
        let constraint =
            annotation.bound.map_or_else(get_mixed, |bound| union_from_annotation(&bound.kind, Some(classname)));
        let default = annotation.default.map(|default| union_from_annotation(&default.kind, Some(classname)));

        let definition = GenericTemplate::new(GenericParent::ClassLike(classname), constraint).with_default(default);
        metadata.add_template_type(name, definition);

        let variance = map_variance(annotation.variance);
        if variance.is_readonly() {
            metadata.template_readonly.insert(name);
        }
        variances.push(variance);
    }

    metadata.set_template_variance(variances);
}

pub fn scan_extends_offsets(metadata: &mut ClassLikeMetadata, annotations: &[ExtendsAnnotation<'_>]) {
    let classname = metadata.name;
    let is_interface = metadata.kind.is_interface();
    for annotation in annotations {
        let parent = ascii_lowercase_word(annotation.r#type.kind.identifier().value);
        let has_parent = if is_interface {
            metadata.all_parent_interfaces.contains(&parent)
        } else {
            metadata.all_parent_classes.contains(&parent)
        };
        if !has_parent || annotation.r#type.type_arguments.is_empty() {
            continue;
        }

        let parameters: Vec<TUnion> = annotation
            .r#type
            .type_arguments
            .iter()
            .map(|argument| union_from_annotation(argument, Some(classname)))
            .collect();
        metadata.template_type_extends_count.insert(parent, parameters.len());
        metadata.add_template_extended_offset(parent, parameters);
    }
}

pub fn scan_implements_offsets(metadata: &mut ClassLikeMetadata, annotations: &[ImplementsAnnotation<'_>]) {
    let classname = metadata.name;
    for annotation in annotations {
        let parent = ascii_lowercase_word(annotation.r#type.kind.identifier().value);
        if !metadata.all_parent_interfaces.contains(&parent) || annotation.r#type.type_arguments.is_empty() {
            continue;
        }

        let parameters: Vec<TUnion> = annotation
            .r#type
            .type_arguments
            .iter()
            .map(|argument| union_from_annotation(argument, Some(classname)))
            .collect();
        metadata.template_type_implements_count.insert(parent, parameters.len());
        metadata.add_template_extended_offset(parent, parameters);
    }
}

pub fn scan_type_aliases(
    metadata: &mut ClassLikeMetadata,
    type_aliases: &[TypeAliasAnnotation<'_>],
    imported: &[ImportedTypeAliasAnnotation<'_>],
) {
    let classname = metadata.name;

    for annotation in imported {
        let fqcn = ascii_lowercase_word(annotation.from.value);
        let type_name = word(annotation.name.value);
        let alias = annotation.r#as.map_or(type_name, |alias| word(alias.value));
        if fqcn == classname {
            continue;
        }

        metadata.imported_type_aliases.insert(alias, (fqcn, type_name, annotation.span));
    }

    for annotation in type_aliases {
        let alias_name = word(annotation.name.value);
        let type_metadata = type_metadata_from_annotation(annotation.r#type, Some(classname));
        metadata.type_aliases.insert(alias_name, type_metadata);
    }
}

pub fn scan_mixins(metadata: &mut ClassLikeMetadata, mixins: &[MixinAnnotation<'_>]) {
    let classname = metadata.name;
    for annotation in mixins {
        metadata.mixins.push(union_from_annotation(&annotation.r#type, Some(classname)));
    }
}

pub fn scan_use_offsets(metadata: &mut ClassLikeMetadata, trait_uses: &[TraitUse<'_>]) {
    let classname = metadata.name;
    for trait_use in trait_uses {
        for annotation in trait_use.use_annotation {
            let parent = ascii_lowercase_word(annotation.r#type.kind.identifier().value);
            if !metadata.used_traits.contains(&parent) || annotation.r#type.type_arguments.is_empty() {
                continue;
            }

            let parameters: Vec<TUnion> = annotation
                .r#type
                .type_arguments
                .iter()
                .map(|argument| union_from_annotation(argument, Some(classname)))
                .collect();
            metadata.template_type_uses_count.insert(parent, parameters.len());
            metadata.add_template_extended_offset(parent, parameters);
        }
    }
}
