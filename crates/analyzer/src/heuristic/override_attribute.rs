use mago_atom::ascii_lowercase_atom;
use mago_codex::get_class_like;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_fixer::SafetyClassification;
use mago_reporting::*;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::code::IssueCode;
use crate::context::Context;

pub fn check_override_attribute<'ctx, 'arena>(
    metadata: &'ctx ClassLikeMetadata,
    members: &[ClassLikeMember<'arena>],
    context: &mut Context<'ctx, 'arena>,
) {
    let class_name = metadata.original_name;
    for member in members {
        let ClassLikeMember::Method(method) = member else {
            continue;
        };

        let (override_attribute, attribute_list_index) = 'outer: {
            for (index, attribute_list) in method.attribute_lists.iter().enumerate() {
                for attribute in attribute_list.attributes.iter() {
                    let fqcn = context.resolved_names.get(&attribute.name);

                    if fqcn.eq_ignore_ascii_case("Override") {
                        break 'outer (Some(attribute), index);
                    }
                }
            }

            (None, 0)
        };

        let name = method.name.value.to_lowercase();
        if name.eq_ignore_ascii_case("__construct") {
            if let Some(attribute) = override_attribute {
                let issue = Issue::error("Invalid `#[Override]` attribute on constructor.")
                    .with_code(IssueCode::InvalidOverrideAttribute)
                    .with_annotation(
                        Annotation::primary(attribute.span())
                            .with_message("Constructors cannot be marked with `#[Override]`."),
                    )
                    .with_note("PHP constructors don't override parent constructors.")
                    .with_help("Remove the `#[Override]` attribute from the constructor.");

                context.collector.propose(issue, |plan| {
                    let attribute_list = &method.attribute_lists.as_slice()[attribute_list_index];
                    if attribute_list.attributes.len() == 1 {
                        plan.delete(attribute_list.span().to_range(), SafetyClassification::Safe);
                    } else {
                        plan.delete(attribute.span().to_range(), SafetyClassification::Safe);
                    }
                });
            }

            continue;
        }

        let lowercase_name = ascii_lowercase_atom(method.name.value);
        let Some(parent_class_names) = metadata.overridden_method_ids.get(&lowercase_name) else {
            if let Some(attribute) = override_attribute {
                let issue = Issue::error(format!("Invalid `#[Override]` attribute on `{class_name}::{name}`."))
                    .with_code(IssueCode::InvalidOverrideAttribute)
                    .with_annotation(
                        Annotation::primary(attribute.span())
                            .with_message("This method doesn't override any parent method."),
                    )
                    .with_note("The attribute should only be used when explicitly overriding a parent method.")
                    .with_help(format!("Remove the `#[Override]` attribute from `{name}` or verify inheritance."));

                context.collector.propose(issue, |plan| {
                    let attribute_list = &method.attribute_lists.as_slice()[attribute_list_index];
                    if attribute_list.attributes.len() == 1 {
                        plan.delete(attribute_list.span().to_range(), SafetyClassification::Safe);
                    } else {
                        plan.delete(attribute.span().to_range(), SafetyClassification::Safe);
                    }
                });
            }

            continue;
        };

        if override_attribute.is_some() || metadata.kind.is_trait() {
            continue;
        }

        let Some(parents_metadata) =
            parent_class_names.iter().filter_map(|parent_class| get_class_like(context.codebase, parent_class)).next()
        else {
            continue;
        };

        let parent_classname = parents_metadata.original_name;

        let issue =
            Issue::error(format!("Missing `#[Override]` attribute on overriding method `{class_name}::{name}`."))
                .with_code(IssueCode::MissingOverrideAttribute)
                .with_annotation(
                    Annotation::primary(method.name.span)
                        .with_message(format!("This method overrides `{parent_classname}::{name}`.")),
                )
                .with_note("The `#[Override]` attribute clarifies intent and prevents accidental signature mismatches.")
                .with_help("Add `#[Override]` attribute to method declaration.");

        context.collector.propose(issue, |plan| {
            let offset = method.span().start.offset;
            let line_start_offset =
                context.source_file.get_line_start_offset(context.source_file.line_number(offset)).unwrap_or(offset);

            let indent = context.source_file.contents[line_start_offset as usize..offset as usize]
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();

            plan.insert(method.span().start.offset, format!("#[\\Override]\n{indent}"), SafetyClassification::Safe);
        });
    }
}
