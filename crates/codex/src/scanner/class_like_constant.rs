use mago_atom::Atom;
use mago_atom::atom;
use mago_names::scope::NamespaceScope;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::ClassLikeConstant;

use crate::issue::ScanningIssueKind;
use crate::metadata::class_like::ClassLikeMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::scanner::Context;
use crate::scanner::attribute::scan_attribute_lists;
use crate::scanner::docblock::ConstantDocblockComment;
use crate::scanner::inference::infer;
use crate::scanner::ttype::get_type_metadata_from_hint;
use crate::scanner::ttype::get_type_metadata_from_type_string;
use crate::ttype::resolution::TypeResolutionContext;
use crate::visibility::Visibility;

#[inline]
pub fn scan_class_like_constants<'ctx, 'arena>(
    class_like_metadata: &mut ClassLikeMetadata,
    constant: &'arena ClassLikeConstant<'arena>,
    classname: Option<Atom>,
    type_context: &TypeResolutionContext,
    context: &mut Context<'ctx, 'arena>,
    scope: &NamespaceScope,
) -> Vec<ClassLikeConstantMetadata> {
    let attributes = scan_attribute_lists(&constant.attribute_lists, context);
    let visibility =
        constant.modifiers.get_first_visibility().and_then(|m| Visibility::try_from(m).ok()).unwrap_or_default();
    let is_final = constant.modifiers.contains_final();
    let type_declaration =
        constant.hint.as_ref().map(|h| get_type_metadata_from_hint(h, Some(class_like_metadata.name), context));

    let mut flags = if is_final { MetadataFlags::FINAL } else { MetadataFlags::empty() };
    if context.file.file_type.is_host() {
        flags |= MetadataFlags::USER_DEFINED;
    } else if context.file.file_type.is_builtin() {
        flags |= MetadataFlags::BUILTIN;
    }

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
            let mut meta = ClassLikeConstantMetadata::new(atom(item.name.value), item.span(), visibility, flags);
            if let Some(type_declaration) = type_declaration.as_ref().cloned() {
                meta.set_type_declaration(type_declaration);
            }

            meta.attributes = attributes.clone();
            meta.inferred_type = infer(context, scope, &item.value).map(|u| u.get_single_owned());

            if let Some(ref docblock) = docblock {
                if docblock.is_deprecated {
                    meta.flags |= MetadataFlags::DEPRECATED;
                }

                if docblock.is_internal {
                    meta.flags |= MetadataFlags::INTERNAL;
                }

                if docblock.is_final {
                    meta.flags |= MetadataFlags::FINAL;
                }

                if let Some(type_string) = &docblock.type_string {
                    match get_type_metadata_from_type_string(type_string, classname, type_context, scope) {
                        Ok(type_metadata) => {
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
