use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeConstant;

use crate::issue::ScanningIssueKind;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::ConstantDocblockComment;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_hint;
use crate::visibility::Visibility;

#[inline]
pub fn scan_class_like_constants(
    class_like_metadata: &mut ClassLikeMetadata,
    constant: &ClassLikeConstant,
    context: &mut Context<'_>,
) -> Vec<ClassLikeConstantMetadata> {
    let attributes = scan_attribute_lists(&constant.attribute_lists, context);
    let visibility =
        constant.modifiers.get_first_visibility().and_then(|m| Visibility::try_from(m).ok()).unwrap_or_default();
    let is_final = constant.modifiers.contains_final();
    let type_metadata =
        constant.hint.as_ref().map(|h| get_type_metadata_from_hint(h, Some(&class_like_metadata.name), context));

    let docblock = match ConstantDocblockComment::create(context, constant) {
        Ok(docblock) => docblock,
        Err(parse_error) => {
            class_like_metadata.issues.push(
                Issue::error("Failed to parse constant docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );

            None
        }
    };

    constant
        .items
        .iter()
        .map(|item| {
            let mut meta = ClassLikeConstantMetadata::new(item.name.value, item.span(), visibility);
            meta.attributes = attributes.clone();
            meta.type_metadata = type_metadata.clone();
            meta.is_final = is_final;
            meta.inferred_type =
                infer(context.interner, context.resolved_names, &item.value).map(|u| u.get_single_owned());

            if let Some(ref docblock) = docblock {
                meta.is_deprecated = docblock.is_deprecated;
                meta.is_internal = docblock.is_internal;
                meta.is_final = docblock.is_final;
            }

            meta
        })
        .collect()
}
