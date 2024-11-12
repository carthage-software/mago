use fennec_ast::ClassLikeConstant;
use fennec_ast::ClassLikeMember;
use fennec_ast::EnumCase;
use fennec_ast::Method;
use fennec_ast::Property;
use fennec_ast::Sequence;
use fennec_ast::TraitUse;
use fennec_span::HasSpan;
use fennec_span::Span;

use crate::array;
use crate::default_line;
use crate::document::Document;
use crate::format::Format;
use crate::group;
use crate::group_break;
use crate::hardline;
use crate::indent;
use crate::settings::BraceStyle;
use crate::softline;
use crate::token;
use crate::Formatter;

pub fn print_class_like_body<'a>(
    f: &mut Formatter<'a>,
    left_brace: &'a Span,
    class_like_members: &'a Sequence<ClassLikeMember>,
    right_brace: &'a Span,
) -> Document<'a> {
    let open = token!(f, *left_brace, "{");

    let mut parts = vec![];

    if let Some(trait_uses) = print_class_like_trait_uses(
        f,
        class_like_members
            .iter()
            .filter_map(|m| match m {
                ClassLikeMember::TraitUse(m) => Some(m),
                _ => None,
            })
            .collect(),
    ) {
        parts.push(trait_uses);
    }

    if let Some(constants) = print_class_like_constants(
        f,
        class_like_members
            .iter()
            .filter_map(|m| match m {
                ClassLikeMember::Constant(m) => Some(m),
                _ => None,
            })
            .collect(),
    ) {
        if !parts.is_empty() {
            parts.push(softline!());
        }

        parts.push(constants);
    }

    if let Some(enum_cases) = print_class_like_enum_cases(
        f,
        class_like_members
            .iter()
            .filter_map(|m| match m {
                ClassLikeMember::EnumCase(m) => Some(m),
                _ => None,
            })
            .collect(),
    ) {
        if !parts.is_empty() {
            parts.push(softline!());
        }

        parts.push(enum_cases);
    }

    let properties = class_like_members.iter().filter_map(|m| match m {
        ClassLikeMember::Property(m) => Some(m),
        _ => None,
    });

    if f.settings.static_properties_first {
        let mut static_properties = vec![];
        let mut non_static_properties = vec![];

        for property in properties {
            if property.modifiers().contains_static() {
                static_properties.push(property);
            } else {
                non_static_properties.push(property);
            }
        }

        if let Some(properties) = print_class_like_properties(f, static_properties) {
            if !parts.is_empty() {
                parts.push(softline!());
            }

            parts.push(properties);
        }

        if let Some(properties) = print_class_like_properties(f, non_static_properties) {
            if !parts.is_empty() {
                parts.push(softline!());
            }

            parts.push(properties);
        }
    } else {
        if let Some(properties) = print_class_like_properties(f, properties.collect()) {
            if !parts.is_empty() {
                parts.push(softline!());
            }

            parts.push(properties);
        }
    }

    let methods = class_like_members.iter().filter_map(|m| match m {
        ClassLikeMember::Method(m) => Some(m),
        _ => None,
    });

    if f.settings.static_methods_first {
        let mut static_methods = vec![];
        let mut non_static_methods = vec![];

        for method in methods {
            if method.modifiers.contains_static() {
                static_methods.push(method);
            } else {
                non_static_methods.push(method);
            }
        }

        if let Some(methods) = print_class_like_methods(f, static_methods) {
            if !parts.is_empty() {
                parts.push(softline!());
            }

            parts.push(methods);
        }

        if let Some(methods) = print_class_like_methods(f, non_static_methods) {
            if !parts.is_empty() {
                parts.push(softline!());
            }

            parts.push(methods);
        }
    } else {
        if let Some(methods) = print_class_like_methods(f, methods.collect()) {
            if !parts.is_empty() {
                parts.push(softline!());
            }

            parts.push(methods);
        }
    }

    if parts.is_empty() {
        return match f.settings.classlike_brace_style {
            BraceStyle::SameLine => {
                group!(open, token!(f, *right_brace, "}"))
            }
            BraceStyle::NextLine => {
                let mut parts = vec![];
                parts.push(open);
                parts.extend(hardline!());
                parts.push(token!(f, *right_brace, "}"));

                group!(@parts)
            }
        };
    }

    group!(open, indent!(array!(softline!(), group_break!(@parts))), default_line!(), token!(f, *right_brace, "}"))
}

fn print_class_like_trait_uses<'a>(f: &mut Formatter<'a>, trait_uses: Vec<&'a TraitUse>) -> Option<Document<'a>> {
    let len = trait_uses.len();
    if len == 0 {
        return None;
    }

    // Format each trait use
    let mut uses_with_docs: Vec<(Span, Document<'a>)> = Vec::with_capacity(len);
    for trait_use in trait_uses {
        let doc = trait_use.format(f);
        uses_with_docs.push((trait_use.span(), doc));
    }

    Some(print_formatted_members(f, uses_with_docs))
}

fn print_class_like_constants<'a>(
    f: &mut Formatter<'a>,
    constants: Vec<&'a ClassLikeConstant>,
) -> Option<Document<'a>> {
    let len = constants.len();
    if len == 0 {
        return None;
    }

    // Format each constant and collect with its name
    let mut constants_with_docs: Vec<(&'a str, Span, Document<'a>)> = Vec::with_capacity(len);
    for constant in constants {
        let name = f.lookup(&constant.first_item().name.value);
        let doc = constant.format(f);
        constants_with_docs.push((name, constant.span(), doc));
    }

    // Sort the constants based on names
    if f.settings.sort_classlike_constants {
        constants_with_docs.sort_by(|(a_name, _, _), (b_name, _, _)| a_name.cmp(b_name));
    }

    // Collect the documents
    let docs = constants_with_docs.into_iter().map(|(_, span, doc)| (span, doc)).collect();

    Some(print_formatted_members(f, docs))
}

fn print_class_like_enum_cases<'a>(f: &mut Formatter<'a>, enum_cases: Vec<&'a EnumCase>) -> Option<Document<'a>> {
    let len = enum_cases.len();
    if len == 0 {
        return None;
    }

    // Format each enum case and collect with its name
    let mut cases_with_docs: Vec<(&'a str, Span, Document<'a>)> = Vec::with_capacity(len);
    for case in enum_cases {
        let name = f.lookup(&case.item.name().value);
        let doc = case.format(f);
        cases_with_docs.push((name, case.span(), doc));
    }

    // Sort the enum cases based on names
    if f.settings.sort_enum_cases {
        cases_with_docs.sort_by(|(a_name, _, _), (b_name, _, _)| a_name.cmp(b_name));
    }

    // Collect the documents
    let docs = cases_with_docs.into_iter().map(|(_, span, doc)| (span, doc)).collect();

    Some(print_formatted_members(f, docs))
}

fn print_class_like_properties<'a>(f: &mut Formatter<'a>, properties: Vec<&'a Property>) -> Option<Document<'a>> {
    let len = properties.len();
    if len == 0 {
        return None;
    }

    // Format each property and collect with its name
    let mut properties_with_docs: Vec<(&'a str, Span, Document<'a>)> = Vec::with_capacity(len);
    for property in properties {
        let name = f.lookup(&property.first_variable().name);
        let doc = property.format(f);
        properties_with_docs.push((name, property.span(), doc));
    }

    // Sort the properties based on names
    if f.settings.sort_properties {
        properties_with_docs.sort_by(|(a_name, _, _), (b_name, _, _)| a_name.cmp(b_name));
    }

    // Collect the documents
    let docs = properties_with_docs.into_iter().map(|(_, span, doc)| (span, doc)).collect();

    Some(print_formatted_members(f, docs))
}

fn print_class_like_methods<'a>(f: &mut Formatter<'a>, methods: Vec<&'a Method>) -> Option<Document<'a>> {
    let len = methods.len();
    if len == 0 {
        return None;
    }

    // Format each method and collect with its name and is_constructor flag
    let mut methods_with_docs: Vec<(&'a str, bool, Span, Document<'a>)> = Vec::with_capacity(len);
    for method in methods {
        let name = f.lookup(&method.name.value);
        let is_constructor = name.eq_ignore_ascii_case("__construct");
        let doc = method.format(f);
        methods_with_docs.push((name, is_constructor, method.span(), doc));
    }

    // Sort the methods based on settings
    if f.settings.constructor_first || f.settings.sort_methods {
        methods_with_docs.sort_by(|(a_name, a_is_ctor, _, _), (b_name, b_is_ctor, _, _)| {
            if f.settings.constructor_first {
                match (a_is_ctor, b_is_ctor) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => {
                        if f.settings.sort_methods {
                            a_name.cmp(b_name)
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    }
                }
            } else if f.settings.sort_methods {
                a_name.cmp(b_name)
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }

    // Collect the documents
    let docs = methods_with_docs.into_iter().map(|(_, _, span, doc)| (span, doc)).collect();

    Some(print_formatted_members(f, docs))
}

fn print_formatted_members<'a>(f: &mut Formatter<'a>, docs: Vec<(Span, Document<'a>)>) -> Document<'a> {
    let mut parts = vec![];
    let len = docs.len();
    for (i, (node, doc)) in docs.into_iter().enumerate() {
        parts.push(doc);

        if i != (len - 1) {
            parts.push(softline!());
        }

        if f.is_next_line_empty(node) {
            parts.extend(hardline!());
        }
    }

    array!(@parts)
}
