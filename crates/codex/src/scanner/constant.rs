use mago_allocator::Arena;
use mago_docblock::error::ParseError;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Constant;
use mago_syntax::ast::Expression;
use mago_syntax::ast::FunctionCall;
use mago_syntax::ast::Literal;
use mago_word::Word;
use mago_word::ascii_lowercase_constant_name_word;

use crate::issue::ScanningIssueKind;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::ConstantDocblockComment;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_type_string;
use crate::scanner::version_claim::evaluate_version_attributes;
use crate::ttype::resolution::TypeResolutionContext;

#[inline]
pub fn scan_constant<'arena, A>(
    constant: &'arena Constant<'arena>,
    context: &Context<'_, 'arena, A>,
    type_context: &TypeResolutionContext,
    scope: &NamespaceScope,
) -> Vec<ConstantMetadata>
where
    A: Arena,
{
    let verdict = evaluate_version_attributes(&constant.attribute_lists, context, context.php_version);

    let attributes = scan_attribute_lists(&constant.attribute_lists, context);
    let docblock = ConstantDocblockComment::create(context, constant);

    let flags = MetadataFlags::origin_flags(context.file.file_type);

    constant
        .items
        .iter()
        .map(|item| {
            let name = ascii_lowercase_constant_name_word(context.resolved_names.get(&item.name));

            let mut metadata = ConstantMetadata::new(name, item.span(), flags);
            metadata.version_constraint = verdict.constraint.clone();
            metadata.attributes.clone_from(&attributes);
            metadata.inferred_type = infer(context, scope, item.value, None);

            process_constant_docblock(context.arena, &mut metadata, &docblock, None, type_context, scope);

            if metadata.attributes.iter().any(|attr| attr.name.as_bytes().eq_ignore_ascii_case(b"Deprecated")) {
                metadata.flags |= MetadataFlags::DEPRECATED;
            }

            metadata
        })
        .collect()
}

#[inline]
pub fn scan_defined_constant<'arena, A>(
    define: &'arena FunctionCall<'arena>,
    context: &Context<'_, 'arena, A>,
    type_context: &TypeResolutionContext,
    scope: &NamespaceScope,
) -> Option<ConstantMetadata>
where
    A: Arena,
{
    let Expression::Identifier(identifier) = define.function else {
        return None;
    };

    let function_name = identifier.value();
    if function_name != b"define" {
        return None;
    }

    let [first_arg, value_arg] = define.argument_list.arguments.as_slice() else {
        return None;
    };

    let Expression::Literal(Literal::String(name_string)) = first_arg.value() else {
        return None;
    };

    let docblock = ConstantDocblockComment::create(context, define);

    let name = ascii_lowercase_constant_name_word(name_string.value?);
    let flags = MetadataFlags::origin_flags(context.file.file_type);

    let mut metadata = ConstantMetadata::new(name, define.span(), flags);
    metadata.inferred_type = infer(context, scope, value_arg.value(), None);

    process_constant_docblock(context.arena, &mut metadata, &docblock, None, type_context, scope);

    Some(metadata)
}

#[inline]
fn process_constant_docblock<A>(
    arena: &A,
    metadata: &mut ConstantMetadata,
    docblock: &Result<Option<ConstantDocblockComment>, ParseError>,
    classname: Option<Word>,
    type_context: &TypeResolutionContext,
    scope: &NamespaceScope,
) where
    A: Arena,
{
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

    if docblock.is_experimental {
        metadata.flags |= MetadataFlags::EXPERIMENTAL;
    }

    if let Some(type_string) = &docblock.type_string {
        match get_type_metadata_from_type_string(arena, type_string, classname, type_context, scope) {
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
