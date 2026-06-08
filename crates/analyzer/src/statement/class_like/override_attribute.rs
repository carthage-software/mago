use mago_allocator::Arena;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_php_version::PHPVersion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeMember;
use mago_text_edit::TextEdit;
use mago_word::ascii_lowercase_word;

use crate::code::IssueCode;
use crate::context::Context;

pub fn check_override_attribute<'ctx, 'arena, A>(
    metadata: &'ctx ClassLikeMetadata,
    members: &[ClassLikeMember<'arena>],
    context: &mut Context<'ctx, 'arena, A>,
) where
    A: Arena,
{
    if context.settings.version < PHPVersion::PHP83 {
        // Override attribute not supported before PHP 8.3
        return;
    }

    let class_name = metadata.original_name;
    for member in members {
        let ClassLikeMember::Method(method) = member else {
            continue;
        };

        let (override_attribute, attribute_list_index) = 'outer: {
            for (index, attribute_list) in method.attribute_lists.iter().enumerate() {
                for attribute in &attribute_list.attributes {
                    let fqcn = context.resolved_names.get(&attribute.name);

                    if fqcn.eq_ignore_ascii_case(b"Override") {
                        break 'outer (Some(attribute), index);
                    }
                }
            }

            (None, 0)
        };

        let name_bytes = method.name.value.to_ascii_lowercase();
        let name = mago_bytes::BytesDisplay(&name_bytes);
        if name_bytes.eq_ignore_ascii_case(b"__construct") {
            if let Some(attribute) = override_attribute {
                let issue = Issue::error("Invalid `#[Override]` attribute on constructor.")
                    .with_code(IssueCode::InvalidOverrideAttribute)
                    .with_annotation(
                        Annotation::primary(attribute.span())
                            .with_message("Constructors cannot be marked with `#[Override]`."),
                    )
                    .with_note("PHP constructors don't override parent constructors.")
                    .with_help("Remove the `#[Override]` attribute from the constructor.");

                context.collector.propose(issue, |edits| {
                    let attribute_list = &method.attribute_lists.as_slice()[attribute_list_index];
                    if attribute_list.attributes.len() == 1 {
                        edits.push(TextEdit::delete(attribute_list.span()));
                    } else {
                        edits.push(TextEdit::delete(attribute.span()));
                    }
                });
            }

            continue;
        }

        let lowercase_name = ascii_lowercase_word(method.name.value);
        let Some(parent_class_names) = metadata.overridden_method_ids.get(&lowercase_name) else {
            if let Some(attribute) = override_attribute {
                let mut issue = Issue::error(format!("Invalid `#[Override]` attribute on `{class_name}::{name}`."))
                    .with_code(IssueCode::InvalidOverrideAttribute)
                    .with_annotation(
                        Annotation::primary(attribute.span())
                            .with_message("This method doesn't override any parent method."),
                    )
                    .with_note("The attribute should only be used when explicitly overriding a parent method.")
                    .with_help(format!("Remove the `#[Override]` attribute from `{name}` or verify inheritance."));

                if metadata.kind.is_trait() {
                    issue = issue.with_note(
                        "If this method is intended to override an interface method, add a `@require-implements` annotation to the trait."
                    );
                }

                context.collector.propose(issue, |edits| {
                    let attribute_list = &method.attribute_lists.as_slice()[attribute_list_index];
                    if attribute_list.attributes.len() == 1 {
                        edits.push(TextEdit::delete(attribute_list.span()));
                    } else {
                        edits.push(TextEdit::delete(attribute.span()));
                    }
                });
            }

            continue;
        };

        if override_attribute.is_some() || metadata.kind.is_trait() {
            continue;
        }

        let has_non_pseudo_parent_method = parent_class_names.values().any(|parent_method_id| {
            let parent_class_name = parent_method_id.get_class_name();
            let method_name = parent_method_id.get_method_name();

            context.codebase.get_class_like(parent_class_name.as_bytes()).is_some_and(|parent_metadata| {
                !parent_metadata.pseudo_methods.contains(&method_name)
                    && !parent_metadata.static_pseudo_methods.contains(&method_name)
            })
        });

        if !has_non_pseudo_parent_method {
            continue;
        }

        let Some(parents_metadata) = parent_class_names
            .values()
            .find_map(|parent_method_id| context.codebase.get_class_like(parent_method_id.get_class_name().as_bytes()))
        else {
            continue;
        };

        let parent_classname = parents_metadata.original_name;

        let original_method_name = mago_bytes::BytesDisplay(method.name.value);

        let issue = Issue::error(format!(
            "Missing `#[Override]` attribute on overriding method `{class_name}::{original_method_name}`."
        ))
        .with_code(IssueCode::MissingOverrideAttribute)
        .with_annotation(
            Annotation::primary(method.name.span)
                .with_message(format!("This method overrides `{parent_classname}::{original_method_name}`.")),
        )
        .with_note("The `#[Override]` attribute clarifies intent and prevents accidental signature mismatches.")
        .with_help("Add `#[Override]` attribute to method declaration.");

        context.collector.propose(issue, |edits| {
            let offset = method.span().start.offset;
            let line_start_offset =
                context.source_file.get_line_start_offset(context.source_file.line_number(offset)).unwrap_or(offset);

            let line_slice = &context.source_file.contents[line_start_offset as usize..offset as usize];
            let indent_end = line_slice.iter().take_while(|b| b.is_ascii_whitespace()).count();
            let indent = std::str::from_utf8(&line_slice[..indent_end]).map(str::to_string).unwrap_or_default();

            edits.push(TextEdit::insert(method.start_offset(), format!("#[\\Override]\n{indent}")));
        });
    }
}
