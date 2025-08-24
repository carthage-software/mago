use mago_atom::ascii_lowercase_constant_name_atom;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::issue::ScanningIssueKind;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::ConstantDocblockComment;
use crate::scanner::inference::infer;

#[inline]
pub fn scan_constant<'ctx, 'ast, 'arena>(
    constant: &'ast Constant<'arena>,
    context: &mut Context<'ctx, 'ast, 'arena>,
) -> Vec<ConstantMetadata> {
    let attributes = scan_attribute_lists(&constant.attribute_lists, context);
    let docblock = ConstantDocblockComment::create(context, constant);

    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    constant
        .items
        .iter()
        .map(|item| {
            let name = ascii_lowercase_constant_name_atom(context.resolved_names.get(&item.name));

            let mut metadata = ConstantMetadata::new(name, item.span(), flags);
            metadata.attributes = attributes.clone();
            metadata.inferred_type = infer(context.resolved_names, &item.value);

            match &docblock {
                Ok(Some(docblock)) => {
                    if docblock.is_deprecated {
                        metadata.flags |= MetadataFlags::DEPRECATED;
                    }

                    if docblock.is_internal {
                        metadata.flags |= MetadataFlags::INTERNAL;
                    }
                }
                Ok(None) => {
                    // No docblock comment found, continue without it
                }
                Err(parse_error) => {
                    metadata.issues.push(
                        Issue::error("Failed to parse constant docblock comment.")
                            .with_code(ScanningIssueKind::MalformedDocblockComment)
                            .with_annotation(
                                Annotation::primary(parse_error.span()).with_message(parse_error.to_string()),
                            )
                            .with_note(parse_error.note())
                            .with_help(parse_error.help()),
                    );
                }
            }

            metadata
        })
        .collect()
}

#[inline]
pub fn scan_defined_constant<'ctx, 'ast, 'arena>(
    define: &'ast FunctionCall<'arena>,
    context: &mut Context<'ctx, 'ast, 'arena>,
) -> Option<ConstantMetadata> {
    let Expression::Identifier(identifier) = define.function else {
        return None;
    };

    let function_name = identifier.value();
    if function_name != "define" {
        return None;
    }

    let arguments = define.argument_list.arguments.as_slice();
    if arguments.len() != 2 {
        return None;
    }

    let Expression::Literal(Literal::String(name_string)) = arguments[0].value() else {
        return None;
    };

    let name = ascii_lowercase_constant_name_atom(name_string.value?);
    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    let mut metadata = ConstantMetadata::new(name, define.span(), flags);
    metadata.inferred_type = infer(context.resolved_names, arguments[1].value());

    Some(metadata)
}
