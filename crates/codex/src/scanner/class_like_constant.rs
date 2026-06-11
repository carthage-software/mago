use mago_allocator::Arena;
use mago_names::scope::NamespaceScope;
use mago_phpdoc_syntax::cst::TagValue;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::cst::ClassLikeConstant;
use mago_syntax::cst::ModifierSequenceExt;
use mago_word::Word;
use mago_word::word;

use crate::issue::ScanningIssueKind;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::find_most_trusted_tag;
use crate::scanner::docblock::parse_docblock;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_hint;
use crate::scanner::ttype::get_type_metadata_from_type;
use crate::scanner::ttype::merge_type_preserving_nullability;
use crate::scanner::version_claim::evaluate_version_attributes;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::reference::TReference;
use crate::ttype::atomic::reference::TReferenceMemberSelector;
use crate::ttype::resolution::TypeResolutionContext;
use crate::visibility::Visibility;

use super::super::ttype::union::TUnion;

#[inline]
pub fn scan_class_like_constants<'arena, A>(
    class_like_metadata: &mut ClassLikeMetadata,
    constant: &'arena ClassLikeConstant<'arena>,
    classname: Option<Word>,
    type_context: &TypeResolutionContext,
    context: &mut Context<'_, 'arena, A>,
    scope: &NamespaceScope,
) -> Vec<ClassLikeConstantMetadata>
where
    A: Arena,
{
    let verdict = evaluate_version_attributes(&constant.attribute_lists, context, context.php_version);

    let attributes = scan_attribute_lists(&constant.attribute_lists, context);
    let visibility =
        constant.modifiers.get_first_visibility().and_then(|m| Visibility::try_from(m).ok()).unwrap_or_default();
    let is_final = constant.modifiers.contains_final();
    let type_declaration =
        constant.hint.as_ref().map(|h| get_type_metadata_from_hint(h, Some(class_like_metadata.name), context));

    let mut flags = if is_final { MetadataFlags::FINAL } else { MetadataFlags::empty() };
    flags |= MetadataFlags::origin_flags(context.file.file_type);

    let document = parse_docblock(context, constant);

    if let Some(document) = document.as_ref() {
        for parse_error in document.errors {
            class_like_metadata.issues.push(
                Issue::error("Failed to parse constant docblock comment.")
                    .with_code(ScanningIssueKind::MalformedDocblockComment)
                    .with_annotation(Annotation::primary(parse_error.span()).with_message(parse_error.to_string()))
                    .with_note(parse_error.note())
                    .with_help(parse_error.help()),
            );
        }
    }

    constant
        .items
        .iter()
        .map(|item| {
            let mut meta = ClassLikeConstantMetadata::new(word(item.name.value), item.span(), visibility, flags);
            meta.version_constraint = verdict.constraint.clone();
            if let Some(type_declaration) = type_declaration.clone() {
                meta.set_type_declaration(type_declaration);
            }

            meta.attributes.clone_from(&attributes);
            meta.inferred_type = infer(context, scope, item.value, classname).map(TUnion::get_single_owned);

            if let Some(TAtomic::Reference(TReference::Member {
                class_like_name,
                member_selector: TReferenceMemberSelector::Identifier(member_name),
            })) = meta.inferred_type.as_ref()
                && classname.is_some_and(|c| c.as_bytes().eq_ignore_ascii_case(class_like_name.as_bytes()))
                && member_name.as_bytes() == item.name.value
            {
                meta.inferred_type = Some(TAtomic::Never);
            }

            if let Some(document) = document.as_ref() {
                for tag in document.tags() {
                    match &tag.value {
                        TagValue::Deprecated(_) => {
                            meta.flags |= MetadataFlags::DEPRECATED;
                        }
                        TagValue::NotDeprecated(_) => {
                            meta.flags.set(MetadataFlags::DEPRECATED, false);
                        }
                        TagValue::Internal(_) => {
                            meta.flags |= MetadataFlags::INTERNAL;
                        }
                        TagValue::Experimental(_) => {
                            meta.flags |= MetadataFlags::EXPERIMENTAL;
                        }
                        TagValue::Final(_) => {
                            meta.flags |= MetadataFlags::FINAL;
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
                            let real_type = meta.type_declaration.as_ref();
                            let type_metadata = merge_type_preserving_nullability(type_metadata, real_type);

                            meta.type_metadata = Some(type_metadata);
                        }
                        Err(typing_error) => class_like_metadata.issues.push(
                            Issue::error("Could not resolve the type for the @var tag.")
                                .with_code(ScanningIssueKind::InvalidVarTag)
                                .with_annotation(
                                    Annotation::primary(typing_error.span()).with_message(typing_error.to_string()),
                                )
                                .with_note(typing_error.note())
                                .with_help(typing_error.help()),
                        ),
                    }
                }
            }

            meta
        })
        .collect()
}
