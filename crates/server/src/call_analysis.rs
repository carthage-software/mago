//! Call sites and target resolution shared by editor queries.

pub(crate) mod cursor;
pub(crate) mod signature;
pub(crate) mod targets;

use mago_analyzer::artifacts::editor::EditorFileData;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_span::HasSpan;
use mago_span::Position;
use mago_span::Span;
use mago_syntax::cst::Call;
use mago_syntax::cst::Expression;
use mago_word::word;

use crate::file_analysis::FileAnalysis;
use crate::semantic_index::Scope;
use crate::semantic_index::ScopeKind;
use crate::symbol_resolution::methods_at;
use crate::{DocumentSymbolItem, Range, SymbolKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CallKind {
    Function,
    Callable,
    Method,
    StaticMethod,
    Constructor,
}

/// Three byte ranges and a kind; names and scopes already live in FileAnalysis.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CallSite {
    pub selection: Range,
    pub(crate) expression: Range,
    receiver: Range,
    kind: CallKind,
}

impl CallSite {
    pub(crate) fn is_callable(&self) -> bool {
        self.kind == CallKind::Callable
    }

    pub(crate) fn from_expression(expression: &Expression<'_>) -> Option<Self> {
        let (kind, selection, receiver) = match expression {
            Expression::Call(Call::Function(call)) => match call.function.unparenthesized() {
                Expression::Identifier(name) => (CallKind::Function, name.span(), name.span()),
                _ => (CallKind::Callable, call.function.span(), call.function.span()),
            },
            Expression::Call(Call::Method(call)) => (CallKind::Method, call.method.span(), call.object.span()),
            Expression::Call(Call::NullSafeMethod(call)) => (CallKind::Method, call.method.span(), call.object.span()),
            Expression::Call(Call::StaticMethod(call)) => {
                (CallKind::StaticMethod, call.method.span(), call.class.span())
            }
            Expression::Instantiation(call) => (CallKind::Constructor, call.class.span(), call.class.span()),
            _ => return None,
        };
        Some(Self {
            selection: range(selection),
            expression: range(expression.span()),
            receiver: range(receiver),
            kind,
        })
    }
}

pub(crate) fn function_target(
    codebase: &CodebaseMetadata,
    file: &File,
    analysis: &FileAnalysis,
    selection: Range,
) -> Option<FunctionLikeIdentifier> {
    let raw = file.contents.get(selection.start as usize..selection.end as usize)?;
    let (start, end, resolved, imported) = analysis.resolved().at_offset(selection.start)?;
    if (start, end) != (selection.start, selection.end) {
        return None;
    }
    let meta = codebase
        .get_function(resolved)
        .or_else(|| (!imported && !raw.contains(&b'\\')).then(|| codebase.get_function(raw)).flatten())?;
    Some(FunctionLikeIdentifier::Function(meta.name))
}

/// Targets whose source parameters match the call's argument positions.
pub(crate) fn call_targets(
    codebase: &CodebaseMetadata,
    file: &File,
    analysis: &FileAnalysis,
    editor: Option<&EditorFileData>,
    call: &CallSite,
) -> Vec<FunctionLikeIdentifier> {
    if call.kind == CallKind::Callable {
        return Vec::new();
    }
    if call.kind == CallKind::Function {
        return function_target(codebase, file, analysis, call.selection).into_iter().collect();
    }
    let mut keys: Vec<_> = editor
        .into_iter()
        .flat_map(|editor| methods_at(editor, call.selection.start))
        .filter(|row| !row.magic && row.expression_span == (call.expression.start, call.expression.end))
        .map(|row| FunctionLikeIdentifier::from(row.method))
        .collect();
    if !keys.is_empty() {
        keys.sort_unstable();
        keys.dedup();
        return keys;
    }
    let raw = &file.contents[call.receiver.start as usize..call.receiver.end as usize];
    let mut classes = Vec::new();
    if matches!(call.kind, CallKind::Constructor | CallKind::StaticMethod) {
        if let Some((start, end, name, _)) = analysis.resolved().at_offset(call.receiver.start)
            && (start, end) == (call.receiver.start, call.receiver.end)
        {
            classes.push(word(name));
        } else if (raw.eq_ignore_ascii_case(b"self")
            || raw.eq_ignore_ascii_case(b"static")
            || raw.eq_ignore_ascii_case(b"parent"))
            && let Some(class) = lexical_class(codebase, file, analysis, call.selection.start)
        {
            classes.extend(if raw.eq_ignore_ascii_case(b"parent") {
                class.direct_parent_class
            } else {
                Some(class.name)
            });
        }
    }
    if call.kind == CallKind::Method
        && raw == b"$this"
        && binds_this(analysis, call.selection.start)
        && let Some(class) = lexical_class(codebase, file, analysis, call.selection.start)
    {
        classes.push(class.name);
    }
    if let Some(editor) = editor {
        let span = if call.kind == CallKind::Constructor { call.expression } else { call.receiver };
        if let Ok(index) = editor.expression_classes.binary_search_by_key(&(span.start, span.end), |row| row.span) {
            classes.extend_from_slice(&editor.expression_classes[index].classes);
        }
    }
    let name: &[u8] = if call.kind == CallKind::Constructor {
        b"__construct"
    } else {
        &file.contents[call.selection.start as usize..call.selection.end as usize]
    };
    for class in classes {
        let method = codebase.get_declaring_method_identifier(&MethodIdentifier::new(class, word(name)));
        if codebase.get_method_by_id(&method).is_some_and(|meta| !meta.flags.is_magic_method()) {
            keys.push(method.into());
        }
    }
    keys.sort_unstable();
    keys.dedup();
    keys
}

/// Which unnamed calls can reach a set of method declarations: `new` reaches
/// constructors, and invoking a value reaches `__invoke`.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ImplicitCalls {
    constructors: bool,
    invocations: bool,
}

impl ImplicitCalls {
    pub(crate) fn of(database: &impl DatabaseReader, declarations: &[Span]) -> Self {
        let mut calls = Self::default();
        for span in declarations {
            let Ok(file) = database.get_ref(&span.file_id) else { continue };
            let Some(name) = file.contents.get(span.start.offset as usize..span.end.offset as usize) else { continue };
            calls.constructors |= name.eq_ignore_ascii_case(b"__construct");
            calls.invocations |= name.eq_ignore_ascii_case(b"__invoke");
        }
        calls
    }

    pub(crate) fn is_empty(self) -> bool {
        !self.constructors && !self.invocations
    }

    /// Selections of the calls in `file` that reach one of `declarations`.
    /// `certain` leaves out invoked values with a non-callable branch.
    pub(crate) fn ranges<'a>(
        self,
        codebase: &'a CodebaseMetadata,
        file: &'a File,
        analysis: &'a FileAnalysis,
        editor: Option<&'a EditorFileData>,
        declarations: &'a [Span],
        certain: bool,
    ) -> impl Iterator<Item = Range> + 'a {
        let reaches = move |key: FunctionLikeIdentifier| {
            codebase
                .get_function_like(&key)
                .and_then(|meta| meta.name_span)
                .is_some_and(|span| declarations.contains(&span))
        };
        analysis
            .calls
            .iter()
            .filter(move |call| match call.kind {
                CallKind::Constructor => {
                    self.constructors && call_targets(codebase, file, analysis, editor, call).into_iter().any(reaches)
                }
                CallKind::Callable => {
                    self.invocations
                        && editor
                            .and_then(|editor| editor.callable_types.invocation_targets_at_end(call.selection.end))
                            .is_some_and(|(targets, uncertain)| {
                                !(certain && uncertain) && targets.iter().any(|target| reaches(target.key()))
                            })
                }
                CallKind::Function | CallKind::Method | CallKind::StaticMethod => false,
            })
            .map(|call| call.selection)
    }
}

/// Include callable invocations without assuming their source parameter positions.
pub(crate) fn call_targets_with_callables(
    codebase: &CodebaseMetadata,
    file: &File,
    analysis: &FileAnalysis,
    editor: Option<&EditorFileData>,
    call: &CallSite,
) -> Vec<FunctionLikeIdentifier> {
    if call.kind != CallKind::Callable {
        return call_targets(codebase, file, analysis, editor, call);
    }
    editor
        .and_then(|editor| editor.callable_types.invocation_targets_at_end(call.selection.end))
        .filter(|(_, uncertain)| !uncertain)
        .map_or_else(Vec::new, |(targets, _)| {
            let mut keys: Vec<_> = targets.iter().map(|target| target.key()).collect();
            keys.sort_unstable();
            keys.dedup();
            keys
        })
}

pub(crate) fn lexical_class<'a>(
    codebase: &'a CodebaseMetadata,
    file: &File,
    analysis: &FileAnalysis,
    offset: u32,
) -> Option<&'a ClassLikeMetadata> {
    let symbol = lexical_class_symbol(&analysis.document_symbols, &file.contents, offset)?;
    let span = Span::new(file.id, Position::new(symbol.range.start), Position::new(symbol.range.end));
    if let Some((start, end, name, _)) = analysis.resolved().at_offset(symbol.selection_range.start)
        && (start, end) == (symbol.selection_range.start, symbol.selection_range.end)
    {
        codebase.get_class_like(name).filter(|class| class.span == span)
    } else {
        codebase.get_anonymous_class(file, span)
    }
}

fn lexical_class_symbol<'a>(
    symbols: &'a [DocumentSymbolItem],
    source: &[u8],
    offset: u32,
) -> Option<&'a DocumentSymbolItem> {
    for symbol in symbols.iter().filter(|symbol| symbol.range.contains(offset)) {
        if let Some(class) = lexical_class_symbol(&symbol.children, source, offset) {
            return Some(class);
        }
        if !matches!(symbol.kind, SymbolKind::Class | SymbolKind::Interface | SymbolKind::Trait | SymbolKind::Enum) {
            continue;
        }
        let anonymous = source
            .get(symbol.selection_range.start as usize..symbol.selection_range.end as usize)
            .is_some_and(|name| name.eq_ignore_ascii_case(b"class"));
        // Anonymous constructor and attribute arguments still run in the outer scope.
        if !anonymous
            || symbol.children.iter().any(|child| {
                child.range.contains(offset)
                    && matches!(
                        child.kind,
                        SymbolKind::Method | SymbolKind::Constructor | SymbolKind::Property | SymbolKind::Constant
                    )
            })
        {
            return Some(symbol);
        }
    }
    None
}

pub(crate) fn binds_this(analysis: &FileAnalysis, offset: u32) -> bool {
    let Some(mut scope) = scope_at(analysis, offset) else { return false };
    loop {
        if scope.is_static {
            return false;
        }
        match scope.kind {
            ScopeKind::Method | ScopeKind::PropertyHook => return true,
            ScopeKind::File | ScopeKind::Function => return false,
            _ => {}
        }
        let Some(parent) = scope.parent else { return false };
        scope = &analysis.semantic().scopes[parent as usize];
    }
}

pub(crate) fn scope_at(analysis: &FileAnalysis, offset: u32) -> Option<&Scope> {
    let scopes = &analysis.semantic().scopes;
    let mut index = scopes.partition_point(|scope| scope.range.start <= offset).checked_sub(1)?;
    loop {
        let scope = &scopes[index];
        if scope.range.contains(offset) {
            return Some(scope);
        }
        index = scope.parent? as usize;
    }
}

fn range(span: Span) -> Range {
    Range::new(span.start.offset, span.end.offset)
}
