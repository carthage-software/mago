use mago_allocator::Arena;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::Use;
use mago_syntax::ast::UseItems;
use mago_syntax::ast::UseType;
use mago_word::concat_word;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use mago_bytes::BytesDisplay;
use mago_bytes::trim_start_byte;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Use<'arena> {
    fn analyze<'ctx, A>(
        &'ast self,
        context: &mut Context<'ctx, 'arena, A>,
        _block_context: &mut BlockContext<'ctx>,
        _artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError>
    where
        A: Arena,
    {
        match &self.items {
            UseItems::Sequence(sequence) => {
                for item in sequence.items.iter() {
                    let fqn = trim_start_byte(item.name.value(), b'\\');
                    check_class_like_import(context, &item.name, fqn);
                }
            }
            UseItems::TypedSequence(typed_sequence) => {
                for item in typed_sequence.items.iter() {
                    let fqn = trim_start_byte(item.name.value(), b'\\');
                    check_typed_import(context, &item.name, fqn, &typed_sequence.r#type);
                }
            }
            UseItems::TypedList(typed_list) => {
                let prefix = trim_start_byte(typed_list.namespace.value(), b'\\');
                for item in typed_list.items.iter() {
                    let fqn = concat_word!(prefix, b"\\", item.name.value());
                    check_typed_import(context, &item.name, fqn.as_bytes(), &typed_list.r#type);
                }
            }
            UseItems::MixedList(mixed_list) => {
                let prefix = trim_start_byte(mixed_list.namespace.value(), b'\\');
                for maybe_typed_item in mixed_list.items.iter() {
                    let fqn = concat_word!(prefix, b"\\", maybe_typed_item.item.name.value());
                    check_maybe_typed_import(
                        context,
                        &maybe_typed_item.item.name,
                        fqn.as_bytes(),
                        maybe_typed_item.r#type.as_ref(),
                    );
                }
            }
        }

        Ok(())
    }
}

fn check_class_like_import<A>(context: &mut Context<'_, '_, A>, name: &Identifier<'_>, fqn: &[u8])
where
    A: Arena,
{
    if !context.codebase.class_like_exists(fqn) && !context.codebase.namespace_exists(fqn) {
        report_non_existent_import(context, name.span(), fqn, "class, interface, trait, or enum");
    }
}

fn check_typed_import<A>(context: &mut Context<'_, '_, A>, name: &Identifier<'_>, fqn: &[u8], use_type: &UseType<'_>)
where
    A: Arena,
{
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

fn check_maybe_typed_import<A>(
    context: &mut Context<'_, '_, A>,
    name: &Identifier<'_>,
    fqn: &[u8],
    use_type: Option<&UseType<'_>>,
) where
    A: Arena,
{
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

fn report_non_existent_import<A>(context: &mut Context<'_, '_, A>, span: Span, fqn: &[u8], symbol_type: &str)
where
    A: Arena,
{
    let fqn = BytesDisplay(fqn);
    context.collector.report_with_code(
        IssueCode::NonExistentUseImport,
        Issue::error(format!("Imported {symbol_type} `{fqn}` does not exist."))
            .with_annotation(Annotation::primary(span).with_message(format!("`{fqn}` not found")))
            .with_help(format!("Ensure that the {symbol_type} `{fqn}` is defined in the codebase.")),
    );
}
