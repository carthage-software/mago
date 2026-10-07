use std::collections::BTreeMap;

use mago_span::Span;
use mago_word::Word;
use mago_word::WordMap;
use mago_word::WordSet;

use crate::assertion::Assertion;
use crate::metadata::CodebaseMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::GenericParent;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::union_comparator;
use crate::ttype::template::TemplateResult;
use crate::ttype::template::inferred_type_replacer;
use crate::ttype::union::TUnion;

/// Determines whether to inherit a type from the parent based on variance rules.
///
/// This function implements smart type inheritance that respects intentional variance:
/// - For return types (covariant): Child can narrow the type
/// - For parameters (contravariant): Child can widen the type
///
/// # Arguments
///
/// * `parent_native` - The parent's native (non-docblock) type
/// * `parent_docblock` - The parent's docblock type metadata
/// * `child_native` - The child's native type
/// * `child_docblock` - The child's docblock type metadata
/// * `covariant` - `true` for return types, `false` for parameters
/// * `has_explicit_inheritdoc` - Whether child has explicit @inheritDoc
/// * `codebase` - The codebase for type comparison
///
/// # Returns
/// `true` if the parent's docblock type should be inherited, `false` otherwise
fn should_inherit_docblock_type(
    parent_native: Option<&TUnion>,
    parent_docblock: Option<&TypeMetadata>,
    child_native: Option<&TUnion>,
    child_docblock: Option<&TypeMetadata>,
    covariant: bool,
    has_explicit_inheritdoc: bool,
    codebase: &CodebaseMetadata,
) -> bool {
    // Allow re-inheritance when the existing child type was itself inherited (not user-written).
    // Inherited types are marked `inferred: true`; user-written docblock types are `inferred: false`.
    if child_docblock.is_some_and(|m| !m.inferred) {
        return false;
    }

    if parent_docblock.is_none() && parent_native.is_none() && covariant {
        return false;
    }

    if has_explicit_inheritdoc {
        return true;
    }

    if parent_docblock.is_none() {
        return false;
    }

    let Some(child_native) = child_native else {
        return true;
    };

    let Some(parent_native) = parent_native else {
        let Some(parent_docblock) = parent_docblock else {
            return false;
        };

        let parent_docblock_type = &parent_docblock.type_union;

        if covariant {
            let child_contained_in_parent_docblock = union_comparator::is_contained_by(
                codebase,
                child_native,
                parent_docblock_type,
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            );

            return !child_contained_in_parent_docblock;
        }

        let parent_docblock_contained_in_child = union_comparator::is_contained_by(
            codebase,
            parent_docblock_type,
            child_native,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        );

        return !parent_docblock_contained_in_child;
    };

    if covariant {
        let child_contained_in_parent = union_comparator::is_contained_by(
            codebase,
            child_native,
            parent_native,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        );

        let types_equal = union_comparator::is_contained_by(
            codebase,
            parent_native,
            child_native,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) && child_contained_in_parent;

        if types_equal {
            return true;
        }

        if let Some(parent_docblock) = parent_docblock {
            let child_contained_in_parent_docblock = union_comparator::is_contained_by(
                codebase,
                child_native,
                &parent_docblock.type_union,
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            );

            if child_contained_in_parent_docblock {
                return false;
            }

            let docblock_type = if !child_native.accepts_null() && parent_docblock.type_union.has_null() {
                parent_docblock.type_union.to_non_nullable()
            } else {
                parent_docblock.type_union.clone()
            };

            let docblock_compatible_with_child = union_comparator::is_contained_by(
                codebase,
                &docblock_type,
                child_native,
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            );

            return docblock_compatible_with_child;
        }

        false
    } else {
        let parent_contained_in_child = union_comparator::is_contained_by(
            codebase,
            parent_native,
            child_native,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        );

        let types_equal = union_comparator::is_contained_by(
            codebase,
            child_native,
            parent_native,
            false,
            false,
            false,
            &mut ComparisonResult::new(),
        ) && parent_contained_in_child;

        types_equal || !parent_contained_in_child
    }
}

/// Performs docblock inheritance for methods that need it.
///
/// Methods inherit docblock from their parent class/interface/trait if:
/// 1. They have an explicit `@inheritDoc` tag, OR
/// 2. They have NO docblock at all (implicit inheritance)
///
/// Template parameters (e.g., `T`) are substituted with concrete types
/// (e.g., `string` when class implements `Interface<string>`).
///
/// When `safe_symbols` is non-empty, classes where both the child and parent
/// are safe are skipped — their docblock inheritance is unchanged from the
/// previous run.
///
/// When `dirty_classes` is provided, only those classes (and their descendants)
/// are considered — O(dirty + descendants) instead of O(all classes).
pub fn inherit_method_docblocks(
    codebase: &mut CodebaseMetadata,
    safe_symbols: &mago_word::WordSet,
    dirty_classes: Option<&WordSet>,
) {
    let mut inheritance_work: Vec<(Word, Word, Word, Word)> = Vec::new();

    // When dirty_classes is provided, expand to include descendants,
    // then use targeted lookups instead of iterating all class_likes.
    if let Some(dirty) = dirty_classes {
        let mut targets = dirty.clone();
        for class_name in dirty {
            if let Some(descendants) = codebase.all_class_like_descendants.get(class_name) {
                targets.extend(descendants.iter().copied());
            }
        }

        for class_name in &targets {
            if !safe_symbols.is_empty() && safe_symbols.contains(class_name) {
                continue;
            }
            if let Some(class_metadata) = codebase.class_likes.get(class_name) {
                collect_inheritance_work(*class_name, class_metadata, &codebase.class_likes, &mut inheritance_work);
            }
        }
    } else {
        for (class_name, class_metadata) in &codebase.class_likes {
            if !safe_symbols.is_empty() && safe_symbols.contains(class_name) {
                continue;
            }
            collect_inheritance_work(*class_name, class_metadata, &codebase.class_likes, &mut inheritance_work);
        }
    }

    apply_inheritance_work(codebase, inheritance_work);
}

/// Collects inheritance work items for a single class.
fn collect_inheritance_work(
    class_name: Word,
    class_metadata: &crate::metadata::class_like::ClassLikeMetadata,
    class_likes: &WordMap<crate::metadata::class_like::ClassLikeMetadata>,
    inheritance_work: &mut Vec<(Word, Word, Word, Word)>,
) {
    for (method_name, method_ids) in &class_metadata.overridden_method_ids {
        let mut parent_method_id = None;

        let mut current_class = class_metadata.direct_parent_class;
        while let Some(parent_name) = current_class {
            if method_ids.contains_key(&parent_name) {
                parent_method_id = Some((parent_name, *method_name));
                break;
            }
            current_class = class_likes.get(&parent_name).and_then(|m| m.direct_parent_class);
        }

        if parent_method_id.is_none() {
            for interface in &class_metadata.all_parent_interfaces {
                if method_ids.contains_key(interface) {
                    parent_method_id = Some((*interface, *method_name));
                    break;
                }
            }
        }

        if parent_method_id.is_none() {
            for trait_name in &class_metadata.used_traits {
                if method_ids.contains_key(trait_name) {
                    parent_method_id = Some((*trait_name, *method_name));
                    break;
                }
            }
        }

        if parent_method_id.is_none()
            && let Some((declaring_class, method_id)) = method_ids.first()
        {
            parent_method_id = Some((*declaring_class, method_id.get_method_name()));
        }

        if let Some((parent_class, parent_method)) = parent_method_id {
            inheritance_work.push((class_name, *method_name, parent_class, parent_method));
        }
    }
}

/// Sorts and applies docblock inheritance work items.
fn apply_inheritance_work(codebase: &mut CodebaseMetadata, mut inheritance_work: Vec<(Word, Word, Word, Word)>) {
    inheritance_work
        .retain(|(class_name, method_name, _, _)| codebase.function_likes.contains_key(&(*class_name, *method_name)));
    inheritance_work.sort_by_cached_key(|(class_name, _, _, _)| {
        codebase.class_likes.get(class_name).map_or(0, |m| m.all_parent_classes.len() + m.all_parent_interfaces.len())
    });

    for (class_name, method_name, parent_class, parent_method) in inheritance_work {
        let child_method_id = (class_name, method_name);
        let parent_method_id = (parent_class, parent_method);

        let Some(parent_method) = codebase.function_likes.get(&parent_method_id) else {
            continue;
        };

        let parent_return_type = parent_method.return_type_metadata.as_ref();
        let parent_native_return_type = parent_method.return_type_declaration_metadata.as_ref();
        let parent_parameters = &parent_method.parameters;
        let parent_template_types = &parent_method.template_types;
        let parent_thrown_types = &parent_method.thrown_types;
        let parent_assertions = &parent_method.assertions;
        let parent_if_true_assertions = &parent_method.if_true_assertions;
        let parent_if_false_assertions = &parent_method.if_false_assertions;

        let Some(child_class) = codebase.class_likes.get(&class_name) else {
            continue;
        };

        let (
            should_inherit_return,
            params_to_inherit,
            should_inherit_templates,
            should_inherit_thrown,
            should_inherit_assertions,
            should_inherit_if_true_assertions,
            should_inherit_if_false_assertions,
            should_clear_inferred_assertions,
        ) = {
            let Some(child_method) = codebase.function_likes.get(&child_method_id) else {
                continue;
            };

            let has_explicit_inherit_doc = child_method.flags.contains(MetadataFlags::INHERITS_DOCS);

            let should_inherit_return = should_inherit_docblock_type(
                parent_native_return_type.map(|m| &m.type_union),
                parent_return_type.filter(|m| m.from_docblock),
                child_method.return_type_declaration_metadata.as_ref().map(|m| &m.type_union),
                child_method.return_type_metadata.as_ref().filter(|m| m.from_docblock),
                true,
                has_explicit_inherit_doc,
                codebase,
            );

            let params_to_inherit: Vec<bool> = parent_parameters
                .iter()
                .enumerate()
                .map(|(i, parent_param)| {
                    let Some(child_param) = child_method.parameters.get(i) else {
                        return false;
                    };

                    should_inherit_docblock_type(
                        parent_param.type_declaration_metadata.as_ref().map(|m| &m.type_union),
                        parent_param.type_metadata.as_ref().filter(|m| m.from_docblock),
                        child_param.type_declaration_metadata.as_ref().map(|m| &m.type_union),
                        child_param.type_metadata.as_ref().filter(|m| m.from_docblock),
                        false,
                        has_explicit_inherit_doc,
                        codebase,
                    )
                })
                .collect();

            let should_inherit_templates = child_method.template_types.is_empty() && !parent_template_types.is_empty();
            let should_inherit_thrown = child_method.thrown_types.is_empty() && !parent_thrown_types.is_empty();

            let parent_has_any_assertions = !parent_assertions.is_empty()
                || !parent_if_true_assertions.is_empty()
                || !parent_if_false_assertions.is_empty();
            let child_assertions_are_inferred = child_method.assertions_inferred;

            let child_assertions_overridable =
                |child: &BTreeMap<Word, Vec<Assertion>>| -> bool { child.is_empty() || child_assertions_are_inferred };

            let should_inherit_assertions =
                child_assertions_overridable(&child_method.assertions) && !parent_assertions.is_empty();
            let should_inherit_if_true_assertions =
                child_assertions_overridable(&child_method.if_true_assertions) && !parent_if_true_assertions.is_empty();
            let should_inherit_if_false_assertions = child_assertions_overridable(&child_method.if_false_assertions)
                && !parent_if_false_assertions.is_empty();

            let should_clear_inferred_assertions = parent_has_any_assertions && child_assertions_are_inferred;

            (
                should_inherit_return,
                params_to_inherit,
                should_inherit_templates,
                should_inherit_thrown,
                should_inherit_assertions,
                should_inherit_if_true_assertions,
                should_inherit_if_false_assertions,
                should_clear_inferred_assertions,
            )
        };

        if !should_inherit_return
            && !params_to_inherit.iter().any(|inherit| *inherit)
            && !should_inherit_templates
            && !should_inherit_thrown
            && !should_inherit_assertions
            && !should_inherit_if_true_assertions
            && !should_inherit_if_false_assertions
            && !should_clear_inferred_assertions
        {
            continue;
        }

        let parent_template_params = child_class.template_extended_parameters.get(&parent_class);
        let template_result = parent_template_params.map(|parent_params| {
            let mut template_result = TemplateResult::default();
            for (template_name, concrete_type) in parent_params {
                template_result.add_lower_bound(
                    *template_name,
                    GenericParent::ClassLike(parent_class),
                    concrete_type.clone(),
                );
            }
            template_result
        });

        let substitute_type = |metadata: &TypeMetadata| {
            if let Some(template_result) = template_result.as_ref() {
                inferred_type_replacer::replace(&metadata.type_union, template_result, codebase)
            } else {
                metadata.type_union.clone()
            }
        };

        let substituted_return_type = parent_return_type
            .filter(|_| should_inherit_return)
            .map(|metadata| (substitute_type(metadata), metadata.span, metadata.from_docblock));

        let substituted_param_types: Vec<Option<(TUnion, Span, bool)>> = parent_parameters
            .iter()
            .zip(&params_to_inherit)
            .map(|(parameter, inherit)| {
                parameter
                    .type_metadata
                    .as_ref()
                    .filter(|_| *inherit)
                    .map(|metadata| (substitute_type(metadata), metadata.span, metadata.from_docblock))
            })
            .collect();

        let parent_templates_to_apply =
            if should_inherit_templates { Some(parent_template_types.clone()) } else { None };
        let parent_thrown_to_apply = should_inherit_thrown.then(|| {
            parent_thrown_types
                .iter()
                .map(|metadata| TypeMetadata::from_docblock(substitute_type(metadata), metadata.span))
                .collect()
        });

        let resolve_assertions = |assertions: &BTreeMap<Word, Vec<Assertion>>| {
            assertions
                .iter()
                .map(|(name, assertions)| {
                    let resolved = if let Some(template_result) = template_result.as_ref() {
                        assertions.iter().flat_map(|a| a.resolve_templates(codebase, template_result)).collect()
                    } else {
                        assertions.clone()
                    };

                    (*name, resolved)
                })
                .collect()
        };

        let parent_assertions_to_apply =
            if should_inherit_assertions { Some(resolve_assertions(parent_assertions)) } else { None };
        let parent_if_true_assertions_to_apply =
            if should_inherit_if_true_assertions { Some(resolve_assertions(parent_if_true_assertions)) } else { None };
        let parent_if_false_assertions_to_apply = if should_inherit_if_false_assertions {
            Some(resolve_assertions(parent_if_false_assertions))
        } else {
            None
        };

        let narrowed_return = if should_inherit_return
            && let Some((type_union, span, from_docblock)) = substituted_return_type
        {
            let child_native_return =
                codebase.function_likes.get(&child_method_id).and_then(|m| m.return_type_declaration_metadata.as_ref());

            let narrowed_type = if let Some(child_native_return) = child_native_return {
                let child_is_more_specific = union_comparator::is_return_type_contained_by(
                    codebase,
                    &child_native_return.type_union,
                    &type_union,
                    false,
                    &mut ComparisonResult::new(),
                );

                let child_matches_parent_native = parent_native_return_type.is_some_and(|parent_native_return| {
                    union_comparator::is_contained_by(
                        codebase,
                        &child_native_return.type_union,
                        &parent_native_return.type_union,
                        false,
                        false,
                        false,
                        &mut ComparisonResult::new(),
                    ) && union_comparator::is_contained_by(
                        codebase,
                        &parent_native_return.type_union,
                        &child_native_return.type_union,
                        false,
                        false,
                        false,
                        &mut ComparisonResult::new(),
                    )
                });

                if child_is_more_specific && !child_matches_parent_native {
                    child_native_return.type_union.clone()
                } else if !child_native_return.type_union.accepts_null() && type_union.has_null() {
                    type_union.to_non_nullable()
                } else {
                    type_union
                }
            } else {
                type_union
            };

            Some(TypeMetadata { type_union: narrowed_type, span, from_docblock, inferred: true })
        } else {
            None
        };

        let Some(child_method) = codebase.function_likes.get_mut(&child_method_id) else {
            continue;
        };

        if let Some(narrowed_return) = narrowed_return {
            child_method.return_type_metadata = Some(narrowed_return);
        }

        for (i, substituted_param) in substituted_param_types.into_iter().enumerate() {
            if params_to_inherit.get(i).copied() == Some(true)
                && let Some(child_param) = child_method.parameters.get_mut(i)
                && let Some((type_union, span, from_docblock)) = substituted_param
            {
                child_param.type_metadata = Some(TypeMetadata { type_union, span, from_docblock, inferred: true });
            }
        }

        if let Some(parent_templates) = parent_templates_to_apply {
            child_method.template_types = parent_templates;
        }

        if let Some(parent_thrown) = parent_thrown_to_apply {
            child_method.thrown_types = parent_thrown;
        }

        if should_clear_inferred_assertions {
            child_method.assertions.clear();
            child_method.if_true_assertions.clear();
            child_method.if_false_assertions.clear();
            child_method.assertions_inferred = false;
        }

        if let Some(parent_asserts) = parent_assertions_to_apply {
            child_method.assertions = parent_asserts;
        }

        if let Some(parent_true_asserts) = parent_if_true_assertions_to_apply {
            child_method.if_true_assertions = parent_true_asserts;
        }

        if let Some(parent_false_asserts) = parent_if_false_assertions_to_apply {
            child_method.if_false_assertions = parent_false_asserts;
        }
    }
}

pub fn inherit_property_docblocks(
    codebase: &mut CodebaseMetadata,
    safe_symbols: &WordSet,
    dirty_classes: Option<&WordSet>,
) {
    let mut inheritance_work: Vec<(Word, Word, Word)> = Vec::new();

    if let Some(dirty) = dirty_classes {
        let mut targets = dirty.clone();
        for class_name in dirty {
            if let Some(descendants) = codebase.all_class_like_descendants.get(class_name) {
                targets.extend(descendants.iter().copied());
            }
        }

        for class_name in &targets {
            if !safe_symbols.is_empty() && safe_symbols.contains(class_name) {
                continue;
            }

            if let Some(class_metadata) = codebase.class_likes.get(class_name) {
                collect_property_inheritance_work(
                    *class_name,
                    class_metadata,
                    &codebase.class_likes,
                    &mut inheritance_work,
                );
            }
        }
    } else {
        for (class_name, class_metadata) in &codebase.class_likes {
            if !safe_symbols.is_empty() && safe_symbols.contains(class_name) {
                continue;
            }

            collect_property_inheritance_work(
                *class_name,
                class_metadata,
                &codebase.class_likes,
                &mut inheritance_work,
            );
        }
    }

    inheritance_work.sort_by_key(|(class_name, _, _)| {
        codebase.class_likes.get(class_name).map_or(0, |m| m.all_parent_classes.len() + m.all_parent_interfaces.len())
    });

    apply_property_inheritance_work(codebase, inheritance_work);
}

fn collect_property_inheritance_work(
    class_name: Word,
    class_metadata: &crate::metadata::class_like::ClassLikeMetadata,
    class_likes: &WordMap<crate::metadata::class_like::ClassLikeMetadata>,
    inheritance_work: &mut Vec<(Word, Word, Word)>,
) {
    for (property_name, parent_ids) in &class_metadata.overridden_property_ids {
        let mut parent_class = None;
        let mut current = class_metadata.direct_parent_class;
        while let Some(name) = current {
            if parent_ids.contains(&name) {
                parent_class = Some(name);
                break;
            }
            current = class_likes.get(&name).and_then(|m| m.direct_parent_class);
        }
        if parent_class.is_none() {
            for trait_name in &class_metadata.used_traits {
                if parent_ids.contains(trait_name) {
                    parent_class = Some(*trait_name);
                    break;
                }
            }
        }
        if parent_class.is_none()
            && let Some(declaring_class) = parent_ids.iter().next()
        {
            parent_class = Some(*declaring_class);
        }

        if let Some(parent_class) = parent_class {
            inheritance_work.push((class_name, *property_name, parent_class));
        }
    }
}

fn apply_property_inheritance_work(codebase: &mut CodebaseMetadata, inheritance_work: Vec<(Word, Word, Word)>) {
    for (class_name, property_name, parent_class) in inheritance_work {
        let Some(parent_metadata) = codebase.class_likes.get(&parent_class) else { continue };
        let Some(parent_property) = parent_metadata.properties.get(&property_name) else { continue };

        let parent_type = parent_property.type_metadata.as_ref();
        let parent_native = parent_property.type_declaration_metadata.as_ref();
        let Some(parent_docblock) = parent_type.filter(|m| m.from_docblock) else {
            continue;
        };

        let Some(child_metadata) = codebase.class_likes.get(&class_name) else { continue };
        let parent_template_params =
            child_metadata.template_extended_parameters.get(&parent_class).cloned().unwrap_or_default();

        let template_result = if parent_template_params.is_empty() {
            None
        } else {
            let mut result = TemplateResult::default();
            for (template_name, concrete_type) in &parent_template_params {
                result.add_lower_bound(*template_name, GenericParent::ClassLike(parent_class), concrete_type.clone());
            }
            Some(result)
        };

        let mut substituted_type = parent_docblock.type_union.clone();
        if let Some(template_result) = template_result.as_ref() {
            substituted_type = inferred_type_replacer::replace(&substituted_type, template_result, codebase);
        }

        let parent_docblock_span = parent_docblock.span;
        let parent_native_for_check = parent_native.map(|m| m.type_union.clone());
        let parent_docblock_for_check = TypeMetadata::from_docblock(substituted_type.clone(), parent_docblock_span);

        let (child_native, child_docblock_owned) = {
            let Some(child_metadata) = codebase.class_likes.get(&class_name) else { continue };
            let Some(child_property) = child_metadata.properties.get(&property_name) else { continue };

            (
                child_property.type_declaration_metadata.as_ref().map(|m| m.type_union.clone()),
                child_property.type_metadata.clone().filter(|m| m.from_docblock),
            )
        };

        if !should_inherit_docblock_type(
            parent_native_for_check.as_ref(),
            Some(&parent_docblock_for_check),
            child_native.as_ref(),
            child_docblock_owned.as_ref(),
            true,
            false,
            codebase,
        ) {
            continue;
        }

        let Some(child_metadata) = codebase.class_likes.get_mut(&class_name) else { continue };
        let Some(child_property) = child_metadata.properties.get_mut(&property_name) else { continue };

        let mut inherited = TypeMetadata::from_docblock(substituted_type, parent_docblock_span);
        inherited.inferred = true;
        child_property.type_metadata = Some(inherited);
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use foldhash::HashSet;
    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_php_version::PHPVersion;
    use mago_syntax::parser::parse_file;
    use mago_word::WordSet;
    use mago_word::word;

    use crate::populator::populate_codebase;
    use crate::reference::SymbolReferences;
    use crate::scanner::scan_program;
    use crate::ttype::TType;

    #[test]
    fn method_docblocks_keep_explicit_types_and_inherit_needed_types() {
        let source = b"<?php
class ParentFailure {}
class ChildFailure {}
/** @template T */
class GenericParent {
    /**
     * @param T $value
     * @param array{unused: T} $unused
     * @return T
     * @throws ParentFailure
     * @mago-assert T $value
     */
    public function convert($value, $unused = []) { return $value; }
}
/** @extends GenericParent<string> */
class InheritedChild extends GenericParent {
    public function convert($value) { return $value; }
}
/** @extends GenericParent<string> */
class ExplicitChild extends GenericParent {
    /**
     * @param int $value
     * @return int
     * @throws ChildFailure
     * @mago-assert int $value
     */
    public function convert($value) { return $value; }
}
class NoOverride extends InheritedChild {}
class Grandchild extends InheritedChild {
    public function convert($value) { return $value; }
}
";
        let file = File::ephemeral(Cow::Borrowed(b"docblocks.php"), Cow::Borrowed(source));
        let arena = LocalArena::new();
        let program = parse_file(&arena, &file);
        assert!(!program.has_errors(), "docblock fixture did not parse: {:?}", program.errors);
        let names = NameResolver::new(&arena).resolve(program);
        let mut codebase = scan_program(&arena, &file, program, &names, PHPVersion::LATEST);
        let mut references = SymbolReferences::new();
        populate_codebase(&mut codebase, &mut references, WordSet::default(), HashSet::default());

        let method_name = word("convert");
        for class_name in [word("inheritedchild"), word("grandchild")] {
            let Some(method) = codebase.function_likes.get(&(class_name, method_name)) else {
                panic!("missing inherited method");
            };
            let Some(return_type) = method.return_type_metadata.as_ref() else {
                panic!("missing inherited return type");
            };
            let Some(parameter_type) = method.parameters[0].type_metadata.as_ref() else {
                panic!("missing inherited parameter");
            };
            assert_eq!(return_type.type_union.get_id(), word("string"), "the return template must resolve");
            assert_eq!(parameter_type.type_union.get_id(), word("string"), "the parameter template must resolve");
            assert!(return_type.inferred && return_type.from_docblock, "the return must retain inherited flags");
            assert!(
                parameter_type.inferred && parameter_type.from_docblock,
                "the parameter must retain inherited flags"
            );
            assert_eq!(method.parameters.len(), 1, "inheritance must not add the missing parameter");
            assert_eq!(method.thrown_types.len(), 1, "the child must inherit the parent's throws");
            assert!(!method.assertions.is_empty(), "the child must inherit assertions");
        }

        let Some(explicit) = codebase.function_likes.get(&(word("explicitchild"), method_name)) else {
            panic!("missing override");
        };
        let Some(return_type) = explicit.return_type_metadata.as_ref() else {
            panic!("missing explicit return type");
        };
        let Some(parameter_type) = explicit.parameters[0].type_metadata.as_ref() else {
            panic!("missing explicit parameter");
        };
        assert_eq!(return_type.type_union.get_id(), word("int"), "the explicit return must remain");
        assert_eq!(parameter_type.type_union.get_id(), word("int"), "the explicit parameter must remain");
        assert!(!return_type.inferred && !parameter_type.inferred, "explicit types must keep their flags");
        assert_eq!(explicit.thrown_types.len(), 1, "explicit throws must remain");
        assert_eq!(explicit.thrown_types[0].type_union.get_id(), word("ChildFailure"), "explicit throws must win");
        assert!(
            !codebase.function_likes.contains_key(&(word("nooverride"), method_name)),
            "inheritance must not add a missing method"
        );
    }
}
