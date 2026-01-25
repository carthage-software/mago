use mago_atom::concat_atom;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::Use;
use mago_syntax::ast::UseItems;
use mago_syntax::ast::UseType;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Use<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        _block_context: &mut BlockContext<'ctx>,
        _artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match &self.items {
            UseItems::Sequence(sequence) => {
                for item in sequence.items.iter() {
                    let fqn = item.name.value().trim_start_matches('\\');
                    check_class_like_import(context, &item.name, fqn);
                }
            }
            UseItems::TypedSequence(typed_sequence) => {
                for item in typed_sequence.items.iter() {
                    let fqn = item.name.value().trim_start_matches('\\');
                    check_typed_import(context, &item.name, fqn, &typed_sequence.r#type);
                }
            }
            UseItems::TypedList(typed_list) => {
                let prefix = typed_list.namespace.value().trim_start_matches('\\');
                for item in typed_list.items.iter() {
                    let fqn = concat_atom!(prefix, "\\", item.name.value());
                    check_typed_import(context, &item.name, &fqn, &typed_list.r#type);
                }
            }
            UseItems::MixedList(mixed_list) => {
                let prefix = mixed_list.namespace.value().trim_start_matches('\\');
                for maybe_typed_item in mixed_list.items.iter() {
                    let fqn = concat_atom!(prefix, "\\", maybe_typed_item.item.name.value());
                    check_maybe_typed_import(context, &maybe_typed_item.item.name, &fqn, &maybe_typed_item.r#type);
                }
            }
        }

        Ok(())
    }
}

fn check_class_like_import(context: &mut Context<'_, '_>, name: &Identifier<'_>, fqn: &str) {
    if !context.codebase.class_like_exists(fqn) && !context.codebase.namespace_exists(fqn) {
        report_non_existent_import(context, name.span(), fqn, "class, interface, trait, or enum");
    }
}

fn check_typed_import(context: &mut Context<'_, '_>, name: &Identifier<'_>, fqn: &str, use_type: &UseType<'_>) {
    match use_type {
        UseType::Function(_) => {
            if !context.codebase.function_exists(fqn) {
                report_non_existent_import(context, name.span(), fqn, "function");
            }
        }
        UseType::Const(_) => {
            if !context.codebase.constant_exists(fqn) {
                report_non_existent_import(context, name.span(), fqn, "constant");
            }
        }
    }
}

fn check_maybe_typed_import(
    context: &mut Context<'_, '_>,
    name: &Identifier<'_>,
    fqn: &str,
    use_type: &Option<UseType<'_>>,
) {
    match use_type {
        Some(UseType::Function(_)) => {
            if !context.codebase.function_exists(fqn) {
                report_non_existent_import(context, name.span(), fqn, "function");
            }
        }
        Some(UseType::Const(_)) => {
            if !context.codebase.constant_exists(fqn) {
                report_non_existent_import(context, name.span(), fqn, "constant");
            }
        }
        None => {
            if !context.codebase.class_like_exists(fqn) && !context.codebase.namespace_exists(fqn) {
                report_non_existent_import(context, name.span(), fqn, "class, interface, trait, or enum");
            }
        }
    }
}

fn report_non_existent_import(context: &mut Context<'_, '_>, span: Span, fqn: &str, symbol_type: &str) {
    context.collector.report_with_code(
        IssueCode::NonExistentUseImport,
        Issue::error(format!("Imported {symbol_type} `{fqn}` does not exist."))
            .with_annotation(Annotation::primary(span).with_message(format!("`{fqn}` not found")))
            .with_help(format!("Ensure that the {symbol_type} `{fqn}` is defined in the codebase.")),
    );
}
