use mago_atom::Atom;
use mago_atom::ascii_lowercase_constant_name_atom;
use mago_docblock::error::ParseError;
use mago_names::scope::NamespaceScope;
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
use crate::scanner::ttype::get_type_metadata_from_type_string;
use crate::ttype::resolution::TypeResolutionContext;

#[inline]
pub fn scan_constant<'ctx, 'arena>(
    constant: &'arena Constant<'arena>,
    context: &mut Context<'ctx, 'arena>,
    type_context: TypeResolutionContext,
    scope: &NamespaceScope,
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
            metadata.inferred_type = infer(context, scope, &item.value);

            process_constant_docblock(&mut metadata, &docblock, None, &type_context, scope);

            metadata
        })
        .collect()
}

#[inline]
pub fn scan_defined_constant<'ctx, 'arena>(
    define: &'arena FunctionCall<'arena>,
    context: &mut Context<'ctx, 'arena>,
    type_context: TypeResolutionContext,
    scope: &NamespaceScope,
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

    let docblock = ConstantDocblockComment::create(context, define);

    let name = ascii_lowercase_constant_name_atom(name_string.value?);
    let mut flags = MetadataFlags::empty();
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

    let mut metadata = ConstantMetadata::new(name, define.span(), flags);
    metadata.inferred_type = infer(context, scope, arguments[1].value());

    process_constant_docblock(&mut metadata, &docblock, None, &type_context, scope);

    Some(metadata)
}

#[inline]
fn process_constant_docblock(
    metadata: &mut ConstantMetadata,
    docblock: &Result<Option<ConstantDocblockComment>, ParseError>,
    classname: Option<Atom>,
    type_context: &TypeResolutionContext,
    scope: &NamespaceScope,
) {
    let docblock = match docblock {
        Ok(docblock) => match docblock {
            Some(docblock) => docblock,
            None => {
                // No docblock comment found, return.
                return;
            }
        },
        Err(parse_error) => {
            metadata.issues.push(
                Issue::error("Failed to parse constant docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );

            return;
        }
    };

    if docblock.is_deprecated {
        metadata.flags |= MetadataFlags::DEPRECATED;
    }

    if docblock.is_internal {
        metadata.flags |= MetadataFlags::INTERNAL;
    }

    if let Some(type_string) = &docblock.type_string {
        match get_type_metadata_from_type_string(type_string, classname, type_context, scope) {
            Ok(type_metadata) => {
                metadata.type_metadata = Some(type_metadata);
            }
            Err(typing_error) => metadata.issues.push(
                Issue::error("Could not resolve the type for the @var tag.")
                    .with_code(ScanningIssueKind::InvalidVarTag)
                    .with_annotation(Annotation::primary(typing_error.span()).with_message(typing_error.to_string()))
                    .with_note(typing_error.note())
                    .with_help(typing_error.help()),
            ),
        }
    }
}
