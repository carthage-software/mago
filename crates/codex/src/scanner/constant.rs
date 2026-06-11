use mago_allocator::Arena;
use mago_names::scope::NamespaceScope;
use mago_phpdoc_syntax::cst::Document;
use mago_phpdoc_syntax::cst::TagValue;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::Constant;
use mago_syntax::cst::Expression;
use mago_syntax::cst::FunctionCall;
use mago_syntax::cst::Literal;
use mago_word::Word;
use mago_word::ascii_lowercase_constant_name_word;

use crate::issue::ScanningIssueKind;
use crate::metadata::constant::ConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::find_most_trusted_tag;
use crate::scanner::docblock::parse_docblock;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_type;
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
    let document = parse_docblock(context, constant);

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

            process_constant_docblock(&mut metadata, document.as_ref(), None, type_context, scope);

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

    let document = parse_docblock(context, define);

    let name = ascii_lowercase_constant_name_word(name_string.value?);
    let flags = MetadataFlags::origin_flags(context.file.file_type);

    let mut metadata = ConstantMetadata::new(name, define.span(), flags);
    metadata.inferred_type = infer(context, scope, value_arg.value(), None);

    process_constant_docblock(&mut metadata, document.as_ref(), None, type_context, scope);

    Some(metadata)
}

#[inline]
fn process_constant_docblock(
    metadata: &mut ConstantMetadata,
    document: Option<&Document<'_>>,
    classname: Option<Word>,
    type_context: &TypeResolutionContext,
    scope: &NamespaceScope,
) {
    let Some(document) = document else {
        return;
    };

    for parse_error in document.errors {
        metadata.issues.push(
            Issue::error("Failed to parse constant docblock comment.")
                .with_code(ScanningIssueKind::MalformedDocblockComment)
                .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                .with_note(parse_error.note())
                .with_help(parse_error.help()),
        );
    }

    for tag in document.tags() {
        match &tag.value {
            TagValue::Deprecated(_) => {
                metadata.flags |= MetadataFlags::DEPRECATED;
            }
            TagValue::NotDeprecated(_) => {
                metadata.flags.set(MetadataFlags::DEPRECATED, false);
            }
            TagValue::Internal(_) => {
                metadata.flags |= MetadataFlags::INTERNAL;
            }
            TagValue::Experimental(_) => {
                metadata.flags |= MetadataFlags::EXPERIMENTAL;
            }
            _ => {}
        }
    }

    let var = find_most_trusted_tag(document, |tag| match &tag.value {
        TagValue::Var(var) => Some(*var),
        _ => None,
    });

    if let Some(var) = &var {
        match get_type_metadata_from_type(var.r#type, classname, type_context, scope) {
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
