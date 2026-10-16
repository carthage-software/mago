//! Resolve a completion site's scope once, then select its candidate providers.

use std::borrow::Cow;

use mago_analyzer::artifacts::editor::EditorFileData;
use mago_database::file::File;
use mago_names::scope::NamespaceScope;
use mago_php_version::PHPVersion;
use mago_syntax::token::{Token, TokenKind};
use mago_word::Word;

use crate::call_analysis::targets::token_receiver_classes;
use crate::domain::{CompletionEntry, CompletionInsert};
use crate::file_analysis::FileAnalysis;
use crate::{Server, lookup};

use super::context::{self, Context, NameIntent};
use super::edit::CompletionEdit;
use super::imports::{self, ImportTarget};
use super::rank::RankContext;
use super::{locals, members, native_types, symbols};

pub(super) struct CompletionSite<'a> {
    file: &'a File,
    tokens: &'a [Token<'a>],
    analysis: &'a FileAnalysis,
    offset: u32,
    context: Context<'a>,
    scope: NamespaceScope,
    scope_offset: u32,
    access_class: Option<Word>,
    import: Option<ImportTarget>,
    edit: Option<CompletionEdit>,
}

impl<'a> CompletionSite<'a> {
    pub(super) fn resolve(
        server: &Server,
        file: &'a File,
        tokens: &'a [Token<'a>],
        analysis: &'a FileAnalysis,
        offset: u32,
    ) -> Option<Self> {
        let context = context::classify(file, tokens, offset, server.php_version)?;
        let scope_offset = scope_offset(file, tokens, offset);
        let scope = analysis.scope_at(scope_offset);
        let access_class = server
            .codebase()
            .file_signatures
            .get(&file.id)
            .into_iter()
            .flat_map(|signature| &signature.ast_nodes)
            .filter(|node| !node.is_function)
            .filter_map(|node| server.codebase().get_class_like_by_name(node.name))
            .filter(|class| class.span.start.offset <= offset && offset.saturating_sub(1) < class.span.end.offset)
            .min_by_key(|class| class.span.end.offset - class.span.start.offset)
            .filter(|_| context::has_class_scope(analysis, offset.saturating_sub(1)))
            .map(|class| class.name);
        let import = access_class.is_none().then(|| imports::target(tokens, offset)).flatten();
        if import.as_ref().is_some_and(|import| {
            import.alias
                || import.group_prefix
                || import.keyword_prefix.is_some_and(|prefix| {
                    // Completion edits must stay on the cursor's line.
                    file.contents[prefix.start as usize..prefix.end as usize]
                        .iter()
                        .any(|byte| matches!(byte, b'\r' | b'\n'))
                })
        }) {
            return None;
        }
        let edit = CompletionEdit::at(tokens, offset, context, import.as_ref());
        Some(Self { file, tokens, analysis, offset, context, scope, scope_offset, access_class, import, edit })
    }

    pub(super) fn candidates(&self, server: &mut Server) -> Vec<CompletionEntry> {
        let access_class = self.access_class.and_then(|name| server.codebase().get_class_like_by_name(name));
        if let Some(import) = &self.import {
            return symbols::qualified_items(
                server.codebase(),
                server.php_version,
                &self.scope,
                &import.qualifier,
                &import.prefix,
                NameIntent::Import(import.kind),
                access_class,
                &RankContext::new(self.file, &self.scope),
            );
        }
        match self.context {
            Context::Variable { prefix } => locals::variable_items(self.analysis, self.tokens, self.offset, prefix),
            Context::InstanceMember { receiver_end, prefix, static_access } => {
                let editor = server.editor_file(self.file.id);
                let classes = editor
                    .as_deref()
                    .map_or(Cow::Borrowed(&[][..]), |editor| self.receiver_classes(server, editor, receiver_end));
                members::instance_member_items(server, &classes, prefix, static_access, self.access_class)
            }
            Context::StaticMember { class, prefix } => members::static_member_items(
                server,
                &self.scope,
                class,
                prefix,
                self.access_class,
                crate::call_analysis::binds_this(self.analysis, self.offset.saturating_sub(1)),
            ),
            Context::Qualified { qualifier, prefix, intent } => symbols::qualified_items(
                server.codebase(),
                server.php_version,
                &self.scope,
                qualifier,
                prefix,
                intent,
                access_class,
                &RankContext::new(self.file, &self.scope),
            ),
            Context::Bare { prefix, intent } => symbols::bare_items(
                server.codebase(),
                server.php_version,
                &self.scope,
                self.analysis.semantic(),
                self.scope_offset,
                prefix,
                intent,
                access_class,
                &RankContext::new(self.file, &self.scope),
            ),
        }
    }

    fn receiver_classes<'s>(
        &'s self,
        server: &Server,
        editor: &'s EditorFileData,
        receiver_end: u32,
    ) -> Cow<'s, [Word]> {
        let receiver = self
            .analysis
            .expression_spans
            .iter()
            .filter(|(_, end)| *end == receiver_end)
            .min_by_key(|(start, _)| *start);
        if let Some(receiver) = receiver {
            return Cow::Borrowed(
                editor
                    .expression_classes
                    .binary_search_by_key(receiver, |entry| entry.span)
                    .map_or(&[], |index| &editor.expression_classes[index].classes),
            );
        }
        let Some(receiver) = self.tokens.partition_point(|token| token.start.offset < receiver_end).checked_sub(1)
        else {
            return Cow::Borrowed(&[]);
        };
        match token_receiver_classes(server.codebase(), self.file, self.analysis, editor, self.tokens, receiver) {
            Some(Cow::Borrowed(expression)) => Cow::Borrowed(&expression.classes),
            Some(Cow::Owned(expression)) => Cow::Owned(expression.classes.into_vec()),
            None => Cow::Borrowed(&[]),
        }
    }

    pub(super) fn apply_edit(&self, items: &mut [CompletionEntry]) {
        if let Some(edit) = &self.edit {
            edit.apply(items);
        }
        if let Some(prefix) = self.import.as_ref().and_then(|import| import.keyword_prefix) {
            let prefix = &self.file.contents[prefix.start as usize..prefix.end as usize];
            let separator = if prefix.last().is_some_and(u8::is_ascii_whitespace) { "" } else { " " };
            let prefix = String::from_utf8_lossy(prefix);
            for item in items {
                let text = format!("{prefix}{separator}{}", item.label);
                item.filter_text = Some(text.clone());
                item.insert = Some(CompletionInsert::Text(text));
            }
        }
    }

    pub(super) fn widens_with_typing(&self) -> bool {
        matches!(self.context, Context::Bare { intent: NameIntent::DeclarationKeyword, .. })
    }

    pub(super) fn native_types(&self, server: &Server) -> Vec<CompletionEntry> {
        native_types::items(server, self.file.id, self.file, self.tokens, self.offset, self.context, &self.scope)
    }
}

pub(super) fn allows_completion(tokens: &[Token<'_>], offset: u32) -> bool {
    let Some(token) = tokens[..tokens.partition_point(|token| token.start.offset < offset)].last() else {
        return true;
    };
    let end = token.start.offset + token.value.len() as u32;
    if offset > end {
        return true;
    }
    if token.kind.is_comment()
        || matches!(
            token.kind,
            TokenKind::LiteralString
                | TokenKind::PartialLiteralString
                | TokenKind::StringPart
                | TokenKind::InlineText
                | TokenKind::InlineShebang
                | TokenKind::OpenTag
                | TokenKind::ShortOpenTag
                | TokenKind::EchoTag
                | TokenKind::CloseTag
                | TokenKind::DocumentStart(_)
                | TokenKind::DocumentEnd
        )
    {
        return false;
    }
    offset == end
        || token.kind.is_identifier_maybe_reserved()
        || matches!(
            token.kind,
            TokenKind::QualifiedIdentifier
                | TokenKind::FullyQualifiedIdentifier
                | TokenKind::Variable
                | TokenKind::Whitespace
        )
}

pub(super) fn expects_name(tokens: &[Token<'_>], offset: u32, version: PHPVersion) -> bool {
    let end = tokens.partition_point(|token| token.start.offset < offset);
    let mut previous = tokens[..end].iter().rev().filter(|token| !lookup::is_trivia(token.kind));
    let Some(keyword) = previous.next() else { return false };
    let before = previous.next().map(|token| token.kind);
    if matches!(
        before,
        Some(
            TokenKind::MinusGreaterThan
                | TokenKind::QuestionMinusGreaterThan
                | TokenKind::ColonColon
                | TokenKind::Function
        )
    ) {
        return false;
    }
    match keyword.kind {
        TokenKind::New | TokenKind::Instanceof | TokenKind::Extends | TokenKind::Implements | TokenKind::Insteadof => {
            true
        }
        TokenKind::Use => before != Some(TokenKind::RightParenthesis),
        TokenKind::Function | TokenKind::Const => before == Some(TokenKind::Use),
        TokenKind::Comma => {
            matches!(context::name_intent(tokens, end, version), NameIntent::Interface | NameIntent::Trait)
        }
        _ => false,
    }
}

fn scope_offset(file: &File, tokens: &[Token<'_>], offset: u32) -> u32 {
    // An unfinished last statement can shorten a semicolon namespace's CST span.
    if let Some(namespace) =
        tokens.iter().rposition(|token| token.start.offset < offset && token.kind == TokenKind::Namespace)
    {
        let mut header = tokens[namespace + 1..].iter().filter(|token| !lookup::is_trivia(token.kind));
        if header
            .next()
            .is_some_and(|token| matches!(token.kind, TokenKind::Identifier | TokenKind::QualifiedIdentifier))
            && header.next().is_some_and(|token| token.value == b";" && token.start.offset < offset)
        {
            return tokens[namespace].start.offset;
        }
    }
    offset.min(file.size.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use mago_syntax::settings::LexerSettings;

    use crate::lookup::TokenCache;

    use super::*;

    fn lexed(marked: &str, check: impl FnOnce(&[Token<'_>], u32)) {
        let Some(offset) = marked.find('|') else { panic!("cursor marker missing") };
        let offset = offset as u32;
        let file = File::ephemeral(b"a.php".as_slice().into(), marked.replacen('|', "", 1).into_bytes().into());
        check(&TokenCache::new(LexerSettings::default()).lex(&file), offset);
    }

    #[test]
    fn completion_stays_out_of_strings_comments_and_operator_tokens() {
        for (marked, allowed) in [
            ("<?php $a->|b;", true),
            ("<?php $a?->|b;", true),
            ("<?php A::|b;", true),
            ("<?php $fo|o;", true),
            ("<?php fo|o();", true),
            ("<?php Foo::cl|ass;", true),
            ("<?php \\Foo\\Ba|r;", true),
            ("<?php $a =  | 1;", true),
            ("<?php $a|", true),
            ("<?php 1.|", true),
            ("|<?php", true),
            ("<?php $a-|>b;", false),
            ("<?php $a?|->b;", false),
            ("<?php $a?-|>b;", false),
            ("<?php A:|:b;", false),
            ("<?php $a ==|= 1;", false),
            ("<?php 12|3;", false),
            ("<?php 'fo|o';", false),
            ("<?php 'foo'|;", false),
            ("<?php // fo|o", false),
            ("<?php /* fo|o */", false),
            ("<?p|hp", false),
        ] {
            lexed(marked, |tokens, offset| assert_eq!(allows_completion(tokens, offset), allowed, "{marked}"));
        }
    }

    #[test]
    fn space_trigger_expects_names_only_after_name_keywords() {
        for (marked, expected) in [
            ("<?php new |", true),
            ("<?php throw new |", true),
            ("<?php $x instanceof |", true),
            ("<?php class A extends |", true),
            ("<?php class A implements |", true),
            ("<?php class A implements B, |", true),
            ("<?php interface A extends B, |", true),
            ("<?php class A { use |", true),
            ("<?php class A { use B, |", true),
            ("<?php class A { use B, C { B::f insteadof |", true),
            ("<?php use |", true),
            ("<?php use function |", true),
            ("<?php use const |", true),
            ("<?php final class |", false),
            ("<?php $value = |", false),
            ("<?php $x->new |", false),
            ("<?php $x?->new |", false),
            ("<?php A::new |", false),
            ("<?php class A { function new |", false),
            ("<?php function |", false),
            ("<?php const |", false),
            ("<?php $f = function () use |", false),
            ("<?php f($a, |", false),
            ("<?php |", false),
        ] {
            lexed(marked, |tokens, offset| {
                assert_eq!(expects_name(tokens, offset, PHPVersion::LATEST), expected, "{marked}");
            });
        }
    }
}
