//! Qualify and validate a native method signature before formatting its stub.

use foldhash::HashMap;
use mago_allocator::LocalArena;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::comparator::{ComparisonResult, union_comparator};
use mago_database::file::File;
use mago_names::ResolvedNames;
use mago_php_version::PHPVersion;
use mago_span::HasSpan;
use mago_syntax::cst::*;

use super::generic;
use crate::{Range, Server};

pub(super) fn render(
    server: &Server,
    destination: &ClassLikeMetadata,
    id: MethodIdentifier,
    file: &File,
    names: &ResolvedNames<'_>,
    method: &Method<'_>,
) -> Option<String> {
    let owner = server.codebase().get_class_like_by_name(id.get_class_name())?;
    let metadata = server.codebase().get_method_by_id(&id)?;
    check_contract(server, owner, metadata, method)?;
    let signature = qualified_signature(server, destination, owner, file, names, method)?;
    format_stub(server, destination, method, &signature)
}

fn check_contract(
    server: &Server,
    owner: &ClassLikeMetadata,
    metadata: &FunctionLikeMetadata,
    method: &Method<'_>,
) -> Option<()> {
    let method_metadata = metadata.method_metadata.as_ref()?;
    if owner.kind.is_trait()
        || generic(owner)
        || !metadata.template_types.is_empty()
        || method_metadata.is_final
        || method_metadata.visibility.is_private()
        || !method_metadata.where_constraints.is_empty()
        || !method.is_abstract()
        || !method.attribute_lists.is_empty()
        || method
            .parameter_list
            .parameters
            .iter()
            .any(|parameter| !parameter.attribute_lists.is_empty() || parameter.is_promoted_property())
    {
        return None;
    }
    for parameter in &metadata.parameters {
        if let (Some(default), Some(declared)) = (&parameter.default_type, &parameter.type_declaration_metadata)
            && !default.type_union.is_expandable()
            && !union_comparator::is_contained_by(
                server.codebase(),
                &default.type_union,
                &declared.type_union,
                false,
                false,
                false,
                &mut ComparisonResult::new(),
            )
        {
            return None;
        }
    }
    Some(())
}

fn qualified_signature(
    server: &Server,
    destination: &ClassLikeMetadata,
    owner: &ClassLikeMetadata,
    file: &File,
    names: &ResolvedNames<'_>,
    method: &Method<'_>,
) -> Option<String> {
    let start = method.function.span.start.offset;
    let end = method.body.start_offset();
    let mut replacements = HashMap::default();
    let mut nodes = vec![Node::FunctionLikeParameterList(&method.parameter_list)];
    if let Some(hint) = &method.return_type_hint {
        nodes.push(Node::FunctionLikeReturnTypeHint(hint));
    }
    for parameter in &method.parameter_list.parameters {
        if parameter.default_value.as_ref().is_some_and(|default| !default.value.is_constant(&server.php_version, true))
        {
            return None;
        }
    }
    while let Some(node) = nodes.pop() {
        let replacement = match node {
            Node::Identifier(identifier) => {
                Some((identifier.span(), names.resolve(&identifier.span().start)?.to_vec()))
            }
            Node::Hint(Hint::Self_(keyword)) => Some((keyword.span, owner.original_name.as_bytes().to_vec())),
            Node::Hint(Hint::Parent(keyword)) => Some((
                keyword.span,
                server.codebase().get_class_like_by_name(owner.direct_parent_class?)?.original_name.as_bytes().to_vec(),
            )),
            Node::ConstantAccess(access) => {
                let resolved = names.resolve(&access.name.span().start)?;
                // Qualification must not remove PHP's runtime namespace fallback.
                if access.name.is_local()
                    && !names.is_imported(&access.name.span().start)
                    && resolved != access.name.value()
                {
                    return None;
                }
                server.codebase().get_constant(resolved)?;
                let name = resolved;
                replacements.insert(
                    Range::new(access.start_offset(), access.end_offset()),
                    format!("\\{}", std::str::from_utf8(name).ok()?),
                );
                continue;
            }
            Node::ClassConstantAccess(access) => {
                let class = match access.class {
                    Expression::Identifier(identifier) => names.resolve(&identifier.span().start)?,
                    Expression::Self_(_) => owner.original_name.as_bytes(),
                    Expression::Parent(_) => {
                        server.codebase().get_class_like_by_name(owner.direct_parent_class?)?.original_name.as_bytes()
                    }
                    _ => return None,
                };
                let ClassLikeConstantSelector::Identifier(constant) = &access.constant else {
                    return None;
                };
                if !constant.value.eq_ignore_ascii_case(b"class")
                    && server.codebase().get_enum_case(class, constant.value).is_none()
                {
                    let constant = server.codebase().get_class_constant(class, constant.value)?;
                    if constant.visibility.is_private()
                        || (constant.visibility.is_protected()
                            && !server.codebase().is_instance_of(destination.name.as_bytes(), class))
                    {
                        return None;
                    }
                }
                replacements.insert(
                    Range::new(access.class.start_offset(), access.class.end_offset()),
                    format!("\\{}", std::str::from_utf8(class).ok()?),
                );
                continue;
            }
            Node::LiteralString(literal) if literal.raw.contains(&b'\n') || literal.raw.contains(&b'\r') => {
                return None;
            }
            Node::Expression(expression)
                if !matches!(
                    expression,
                    Expression::Literal(_)
                        | Expression::Array(_)
                        | Expression::LegacyArray(_)
                        | Expression::Binary(_)
                        | Expression::UnaryPrefix(_)
                        | Expression::Parenthesized(_)
                        | Expression::Conditional(_)
                        | Expression::ConstantAccess(_)
                        | Expression::Access(Access::ClassConstant(_))
                ) =>
            {
                return None;
            }
            _ => None,
        };
        if let Some((span, name)) = replacement {
            replacements.insert(
                Range::new(span.start.offset, span.end.offset),
                format!("\\{}", std::str::from_utf8(&name).ok()?),
            );
            continue;
        }
        node.visit_children(|child| nodes.push(child));
    }
    let mut replacements: Vec<_> = replacements.into_iter().collect();
    replacements.sort_unstable_by_key(|(range, _)| range.start);
    let mut signature = String::new();
    let mut cursor = start;
    for (range, text) in replacements {
        if range.start < cursor || range.end > end {
            return None;
        }
        signature.push_str(std::str::from_utf8(&file.contents[cursor as usize..range.start as usize]).ok()?);
        signature.push_str(&text);
        cursor = range.end;
    }
    signature.push_str(std::str::from_utf8(&file.contents[cursor as usize..end as usize]).ok()?);
    Some(signature)
}

fn format_stub(
    server: &Server,
    destination: &ClassLikeMetadata,
    method: &Method<'_>,
    signature: &str,
) -> Option<String> {
    let attribute =
        if server.php_version >= PHPVersion::PHP83 && !method.name.value.eq_ignore_ascii_case(b"__construct") {
            "#[\\Override] "
        } else {
            ""
        };
    let source = format!(
        "<?php {} MagoStub {{ {attribute}public {}{} {{ throw new \\LogicException('Not implemented.'); }} }}",
        if destination.kind.is_enum() { "enum" } else { "class" },
        if method.is_static() { "static " } else { "" },
        signature.trim_end()
    );
    let arena = LocalArena::new();
    let file = File::ephemeral(b"stub.php".as_slice().into(), source.into_bytes().into());
    crate::capabilities::refactor_stub::format_body(server, &arena, &file, server.parser_settings())
}
