//! Resolve call targets from editor data, with token-based fallback for unfinished calls.

use std::borrow::Cow;
use std::sync::Arc;

use mago_analyzer::artifacts::editor::{BoundSignature, CallableTarget, EditorFileData, ExpressionClasses};
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_codex::ttype::union::TUnion;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_names::kind::NameKind;
use mago_syntax::token::Token;
use mago_syntax::token::TokenKind;
use mago_word::Word;

use crate::Server;
use crate::file_analysis::FileAnalysis;
use crate::lookup;
use crate::semantic_index::BindingKind;
use crate::semantic_index::SymbolKind;
use crate::symbol_resolution::methods_at;
use crate::symbol_resolution::parameter_at;

use super::binds_this;
use super::cursor::CallCursor;
use super::cursor::is_name;
use super::lexical_class;

pub(crate) struct SignatureTarget {
    pub key: FunctionLikeIdentifier,
    pub bound: Option<Arc<BoundSignature>>,
    /// The original-case owner displayed by signature help. An inherited
    /// constructor keeps the instantiated class, not its declaring class.
    pub owner: Option<Word>,
    pub name: Option<Word>,
    /// Unknown branches and magic dispatch cannot bind names safely.
    pub uncertain: bool,
}

/// Resolve a cursor against the same trivia-free token slice that produced it.
pub(crate) fn resolve_signature_targets(
    server: &mut Server,
    file_id: FileId,
    tokens: &[Token<'_>],
    call: &CallCursor<'_>,
) -> Vec<SignatureTarget> {
    let name = &tokens[call.name];
    let constructor =
        call.attribute || call.name.checked_sub(1).is_some_and(|index| tokens[index].kind == TokenKind::New);
    let editor = server.editor_file(file_id);
    let mut targets = Vec::new();
    let member = call.name.checked_sub(1).is_some_and(|index| {
        matches!(
            tokens[index].kind,
            TokenKind::ColonColon | TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan
        )
    });
    if !constructor && !member {
        if let Some((keys, uncertain)) =
            editor.as_ref().and_then(|editor| editor.callable_types.at_end(name.start.offset + name.value.len() as u32))
        {
            return keys
                .iter()
                .map(|target| {
                    let bound = match target {
                        CallableTarget::Full(_) => None,
                        CallableTarget::Signature(signature) => Some(Arc::clone(signature)),
                    };
                    let uncertain = uncertain
                        || bound.as_ref().is_some_and(|signature| {
                            signature.parameters.iter().any(|parameter| parameter.name.is_none())
                        });
                    SignatureTarget { key: target.key(), bound, owner: None, name: None, uncertain }
                })
                .collect();
        }
        if !is_name(name.kind) {
            return targets;
        }
    }
    if let Some(editor) = &editor {
        for method in methods_at(editor, name.start.offset) {
            let presentation = if constructor || method.dynamic || method.magic {
                None
            } else {
                server.method_presentation(method.receiver_class, name.value)
            };
            if server.codebase().get_method_by_id(&method.method).is_some() {
                let class = server.codebase().get_class_like(method.method.get_class_name().as_bytes());
                targets.push(SignatureTarget {
                    key: method.method.into(),
                    bound: None,
                    owner: presentation.map(|method| method.owner).or_else(|| class.map(|class| class.original_name)),
                    name: presentation.map(|method| method.name),
                    uncertain: method.uncertain || method.magic,
                });
            }
        }
    }

    if targets.is_empty() {
        let Some(analysis) = server.file_analysis_for(file_id) else { return targets };
        let Ok(file) = server.database().get(&file_id) else { return targets };
        let scope = analysis.scope_at(name.start.offset);
        let previous = call.name.checked_sub(1).map(|index| &tokens[index]);
        let member = previous.is_some_and(|token| {
            matches!(
                token.kind,
                TokenKind::ColonColon | TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan
            )
        });
        if constructor || member {
            let receiver_index = if constructor { Some(call.name) } else { call.name.checked_sub(2) };
            if let Some(receiver_index) = receiver_index {
                let receiver = &tokens[receiver_index];
                let mut classes = Vec::new();
                let mut uncertain = false;
                if constructor || previous.is_some_and(|token| token.kind == TokenKind::ColonColon) {
                    let fetched = receiver_index.checked_sub(1).is_some_and(|index| {
                        matches!(
                            tokens[index].kind,
                            TokenKind::ColonColon | TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan
                        )
                    });
                    if matches!(receiver.kind, TokenKind::Self_ | TokenKind::Static | TokenKind::Parent) {
                        if let Some(class) = lexical_class(server.codebase(), &file, &analysis, name.start.offset) {
                            classes.extend(if receiver.kind == TokenKind::Parent {
                                class.direct_parent_class
                            } else {
                                Some(class.name)
                            });
                        }
                    } else if is_name(receiver.kind) && !fetched {
                        let (name, _) = scope.resolve_str(NameKind::Default, receiver.value);
                        classes.push(mago_word::word(&name));
                    }
                }
                if classes.is_empty()
                    && let Some(editor) = &editor
                {
                    let end = receiver.start.offset + receiver.value.len() as u32;
                    if let Some(expression) = editor
                        .expression_classes
                        .iter()
                        .filter(|entry| entry.span.1 == end)
                        .min_by_key(|entry| entry.span.0)
                    {
                        classes.extend_from_slice(&expression.classes);
                        uncertain |= expression.uncertain;
                    }
                    if classes.is_empty()
                        && let Some(expression) =
                            token_receiver_classes(server.codebase(), &file, &analysis, editor, tokens, receiver_index)
                    {
                        classes.extend_from_slice(&expression.classes);
                        uncertain |= expression.uncertain;
                    }
                }
                for class in classes {
                    let requested = if constructor { b"__construct".as_slice() } else { name.value };
                    let presentation = if constructor { None } else { server.method_presentation(class, requested) };
                    let method = server
                        .codebase()
                        .get_declaring_method_identifier(&MethodIdentifier::new(class, mago_word::word(requested)));
                    if server.codebase().get_method_by_id(&method).is_some() {
                        let class = server.codebase().get_class_like(class.as_bytes());
                        targets.push(SignatureTarget {
                            key: method.into(),
                            bound: None,
                            owner: presentation
                                .map(|method| method.owner)
                                .or_else(|| class.map(|class| class.original_name)),
                            name: presentation.map(|method| method.name),
                            uncertain,
                        });
                    } else {
                        uncertain = true;
                    }
                }
                if uncertain {
                    for target in &mut targets {
                        target.uncertain = true;
                    }
                }
            }
        } else {
            let (resolved, imported) = scope.resolve_str(NameKind::Function, name.value);
            let function = server.codebase().get_function(&resolved).or_else(|| {
                (!imported && name.kind == TokenKind::Identifier)
                    .then(|| server.codebase().get_function(name.value))
                    .flatten()
            });
            if let Some(function) = function {
                targets.push(SignatureTarget {
                    key: FunctionLikeIdentifier::Function(function.name),
                    bound: None,
                    owner: None,
                    name: None,
                    uncertain: false,
                });
            }
        }
    }
    targets
}

/// Classes of a receiver the parser dropped, from the variable, `$this` or
/// property fetch that its tokens spell.
pub(crate) fn token_receiver_classes<'a>(
    codebase: &CodebaseMetadata,
    file: &File,
    analysis: &FileAnalysis,
    editor: &'a EditorFileData,
    tokens: &[Token<'_>],
    receiver: usize,
) -> Option<Cow<'a, ExpressionClasses>> {
    let token = &tokens[receiver];
    let span = (token.start.offset, token.start.offset + token.value.len() as u32);
    let operator = tokens[..receiver].iter().rposition(|token| !lookup::is_trivia(token.kind));
    let operator_kind = operator.map(|index| tokens[index].kind);
    if token.kind == TokenKind::Variable {
        if matches!(
            operator_kind,
            Some(
                TokenKind::ColonColon
                    | TokenKind::MinusGreaterThan
                    | TokenKind::QuestionMinusGreaterThan
                    | TokenKind::Dollar
            )
        ) {
            return None;
        }
        if token.value == b"$this" {
            if !binds_this(analysis, token.start.offset) {
                return None;
            }
            let class = lexical_class(codebase, file, analysis, token.start.offset)?;
            return Some(Cow::Owned(ExpressionClasses { span, classes: Box::new([class.name]), uncertain: false }));
        }
        return previous_receiver_classes(codebase, file, analysis, editor, tokens, token);
    }
    if !token.kind.is_identifier_maybe_reserved()
        || !matches!(operator_kind, Some(TokenKind::MinusGreaterThan | TokenKind::QuestionMinusGreaterThan))
    {
        return None;
    }
    let object = tokens[..operator?].iter().rposition(|token| !lookup::is_trivia(token.kind))?;
    let objects = token_receiver_classes(codebase, file, analysis, editor, tokens, object)?;
    let name = [b"$".as_slice(), token.value].concat();
    let mut classes = Vec::new();
    let mut uncertain = objects.uncertain;
    for &class in &objects.classes {
        let property = match codebase.get_declaring_property_class(class.as_bytes(), &name) {
            Some(declaring) => codebase.get_property(declaring.as_bytes(), &name).map(|property| (declaring, property)),
            None => codebase.get_declaring_magic_property(class.as_bytes(), &name).map(|property| (class, property)),
        };
        let expression = property.and_then(|(declaring, property)| {
            expanded_classes(codebase, span, &property.type_metadata.as_ref()?.type_union, || TypeExpansionOptions {
                self_class: Some(declaring),
                static_class_type: StaticClassType::Name(class),
                ..Default::default()
            })
        });
        match expression {
            Some(expression) => {
                classes.extend_from_slice(&expression.classes);
                uncertain |= expression.uncertain;
            }
            None => uncertain = true,
        }
    }
    if classes.is_empty() {
        return None;
    }
    classes.sort_unstable();
    classes.dedup();
    Some(Cow::Owned(ExpressionClasses { span, classes: classes.into_boxed_slice(), uncertain }))
}

fn expanded_classes(
    codebase: &CodebaseMetadata,
    span: (u32, u32),
    union: &TUnion,
    options: impl FnOnce() -> TypeExpansionOptions,
) -> Option<ExpressionClasses> {
    if !union.is_expandable() {
        return ExpressionClasses::from_union(span, union);
    }
    let mut union = union.clone();
    expand_union(codebase, &mut union, &options());
    ExpressionClasses::from_union(span, &union)
}

fn previous_receiver_classes<'a>(
    codebase: &CodebaseMetadata,
    file: &File,
    analysis: &FileAnalysis,
    editor: &'a EditorFileData,
    tokens: &[Token<'_>],
    receiver: &Token<'_>,
) -> Option<Cow<'a, ExpressionClasses>> {
    let index = analysis.semantic();
    let scope = index.scope_at(receiver.start.offset).map(|scope| scope.range);
    let previous = index.occurrences.iter().rev().find(|entry| {
        let symbol = &index.symbols[entry.symbol as usize];
        entry.range.end <= receiver.start.offset
            && symbol.kind == SymbolKind::Variable
            && Some(symbol.name.as_ref()) == receiver.value.strip_prefix(b"$")
            && index.scope_at(entry.range.start).map(|scope| scope.range) == scope
    })?;
    let next = tokens.partition_point(|token| token.start.offset < previous.range.end);
    let assignment = tokens[next..]
        .iter()
        .find(|token| !lookup::is_trivia(token.kind))
        .is_some_and(|token| token.kind == TokenKind::Equal);
    // A dropped function body must not borrow a variable from its outer scope.
    if tokens[next..].iter().take_while(|token| token.start.offset < receiver.start.offset).any(|token| {
        matches!(token.kind, TokenKind::Function | TokenKind::Fn)
            && index.scope_at(token.start.offset).map(|scope| scope.range) == scope
    }) {
        return None;
    }
    if previous.declaration
        && matches!(
            index.symbols[previous.symbol as usize].binding,
            Some(BindingKind::Parameter | BindingKind::PromotedProperty)
        )
    {
        let (parameter, _) = parameter_at(codebase, file, index, previous.range.start)
            .filter(|(parameter, _)| !parameter.flags.is_variadic())?;
        let union = &parameter.type_metadata.as_ref()?.type_union;
        return expanded_classes(codebase, (previous.range.start, previous.range.end), union, || {
            let class = lexical_class(codebase, file, analysis, previous.range.start).map(|class| class.name);
            TypeExpansionOptions {
                self_class: class,
                static_class_type: class.map_or(StaticClassType::None, StaticClassType::Name),
                ..Default::default()
            }
        })
        .map(Cow::Owned);
    }
    editor
        .expression_classes
        .iter()
        .filter(|entry| entry.span.0 == previous.range.start && (entry.span.1 == previous.range.end || assignment))
        .min_by_key(|entry| entry.span.1)
        .map(Cow::Borrowed)
}
