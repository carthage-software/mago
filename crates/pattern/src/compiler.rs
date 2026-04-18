//! Compile a pattern source string into a [`Pattern<MagoQueryContext>`] the grit engine
//! can execute.
//!
//! The compilation pipeline is standard grit fare:
//!
//! 1. **Substitute** metavariables: `^name` → `µname` so the host parser accepts them as
//!    plain identifiers.
//! 2. **Wrap** the source in a snippet context (try each wrapper from
//!    [`MagoLanguage::snippet_context_strings`] until one parses without errors).
//! 3. **Parse** the wrapped source via `mago-syntax`, the unchanged production parser.
//! 4. **Locate** the inner node that corresponds to the user's original snippet (skip the
//!    wrapper boilerplate).
//! 5. **Lower** that AST node into a [`Pattern<MagoQueryContext>`] tree, replacing any
//!    `µ`-prefixed identifier with a [`Pattern::Variable`] slot.
//!
//! The pattern source grammar for the MVP is a bare PHP snippet: no `where` clauses, no
//! `<:` operator, no `or`/`not` surface yet. Those are GritQL-surface features that will
//! come in a subsequent pass.

use std::borrow::Cow;

use bumpalo::Bump;
use grit_pattern_matcher::constants::DEFAULT_FILE_NAME;
use grit_pattern_matcher::pattern::And;
use grit_pattern_matcher::pattern::Pattern;
use grit_pattern_matcher::pattern::Variable;
use grit_pattern_matcher::pattern::VariableContent;
use grit_pattern_matcher::pattern::VariableSource;
use grit_util::Language;
use mago_database::file::File;
use mago_database::file::FileId;
use mago_span::HasSpan;
use mago_syntax::ast::Expression;
use mago_syntax::ast::Identifier;
use mago_syntax::ast::LocalIdentifier;
use mago_syntax::ast::Node;
use mago_syntax::ast::Program;
use mago_syntax::ast::Variable as PhpVariable;
use mago_syntax::parser::parse_file;

use crate::language::MagoLanguage;
use crate::node_pattern::MagoLeafNodePattern;
use crate::node_pattern::MagoNodePattern;
use crate::query_context::MagoQueryContext;

/// A pattern that has been compiled and is ready to execute against a target file.
pub struct CompiledPattern {
    pub pattern: Pattern<MagoQueryContext>,
    /// Metavariable names captured by the pattern, in slot order.
    pub variables: Vec<String>,
    /// The preprocessed (`µ`-substituted) source that was fed to the parser. Kept for
    /// diagnostics; useful when a query errors out and the engineer wants to see what the
    /// parser actually saw.
    pub preprocessed_source: String,
    /// The chosen snippet context wrapper: the `(prefix, suffix)` pair that successfully
    /// parsed the substituted source.
    pub context: (&'static str, &'static str),
}

/// First index available for pattern-specific variable slots in the global scope. Lower
/// indices are reserved by grit itself for `FILENAME`, `ABSOLUTE_PATH`, `PROGRAM`, `$match`.
const RESERVED_SLOT_COUNT: usize = 4;

#[derive(Debug)]
pub enum CompileError {
    Empty,
    NoContextParses(String),
    UnsupportedPattern(&'static str),
    /// The surface-grammar parser rejected the pattern source.
    SurfaceError(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "pattern source is empty"),
            Self::NoContextParses(errors) => {
                write!(f, "pattern source did not parse under any snippet context: {errors}")
            }
            Self::UnsupportedPattern(reason) => write!(f, "pattern is not yet supported: {reason}"),
            Self::SurfaceError(err) => write!(f, "surface grammar error: {err}"),
        }
    }
}

impl std::error::Error for CompileError {}

/// Interns a metavariable name and returns its slot offset (0-based, not including the
/// four grit-reserved slots at the front of scope 0).
///
/// Exposed for the surface parser so patterns like `^name` at the top level can share the
/// same slot-assignment logic the snippet lowering pass uses.
pub(crate) fn reserve_meta_slot(variables: &mut Vec<String>, name: &str) -> usize {
    if let Some(pos) = variables.iter().position(|v| v == name) {
        return pos;
    }
    let pos = variables.len();
    variables.push(name.to_string());
    pos
}

/// Look up an already-reserved metavariable slot by name. Returns `None` if the name has
/// not been introduced yet. Used on the right-hand side of `=>` rewrites to reject
/// references to variables that the left-hand side didn't bind.
pub(crate) fn lookup_meta_slot(variables: &[String], name: &str) -> Option<usize> {
    variables.iter().position(|v| v == name)
}

/// Compiles a bare snippet source (no backticks, no `where` clauses) into a
/// [`Pattern<MagoQueryContext>`]. Shared between [`compile`] (top-level bare snippet
/// fallback) and the surface parser (for the contents of each backticked snippet).
pub(crate) fn lower_snippet_source(
    arena: &Bump,
    source: &str,
    variables: &mut Vec<String>,
) -> Result<Pattern<MagoQueryContext>, CompileError> {
    let language = MagoLanguage::new();
    let substituted = language.substitute_metavariable_prefix(source);

    for &(prefix, suffix) in language.snippet_context_strings() {
        let mut wrapped = String::with_capacity(prefix.len() + substituted.len() + suffix.len());
        wrapped.push_str(prefix);
        wrapped.push_str(&substituted);
        wrapped.push_str(suffix);

        let user_start = prefix.len() as u32;
        let user_end = (wrapped.len() - suffix.len()) as u32;
        let file = File::ephemeral(Cow::Borrowed("<pattern>"), Cow::Owned(wrapped));
        let program = parse_file(arena, &file);
        if !program.errors.is_empty() {
            continue;
        }
        let Some(root) = find_user_node(program, user_start, user_end) else {
            continue;
        };
        return Ok(lower(&root, program.source_text, variables));
    }

    Err(CompileError::NoContextParses("none of the snippet wrappers produced a clean parse".into()))
}

/// Compiles a pattern source string into a [`CompiledPattern`].
///
/// The source can be either:
/// * A bare PHP snippet with `^name` metavariables, e.g. `eval(^x)`. Backwards-compatible
///   shorthand for patterns that don't need surface-grammar constructs.
/// * A full GritQL-subset surface pattern, e.g. ``\u{0060}eval(^x)\u{0060} where { ^x <: not \u{0060}...\u{0060} }``.
///
/// The caller provides an arena that must outlive the returned pattern: Mago's parsed AST
/// lives in the arena and the compiled pattern references it.
pub fn compile(arena: &Bump, source: &str) -> Result<CompiledPattern, CompileError> {
    let trimmed = source.trim();
    if trimmed.is_empty() {
        return Err(CompileError::Empty);
    }

    let surface = crate::surface::parse(arena, trimmed)?;

    Ok(CompiledPattern {
        pattern: surface.pattern,
        variables: surface.variables,
        preprocessed_source: trimmed.to_string(),
        context: ("", ""),
    })
}

/// Build the engine's variable registry from the metavariable names captured during
/// compilation. Reserved slots are populated with blank `VariableContent` so the grit
/// engine can touch `FILENAME` / `PROGRAM` at the well-known indices.
pub fn build_var_registry<'a>(compiled: &CompiledPattern) -> Vec<Vec<Vec<Box<VariableContent<'a, MagoQueryContext>>>>> {
    let mut scope_0: Vec<Box<VariableContent<'a, MagoQueryContext>>> = Vec::new();
    for reserved in ["$filename", "$absolute_path", "$program", "$match"].iter() {
        scope_0.push(Box::new(VariableContent::new((*reserved).to_string())));
    }
    for name in &compiled.variables {
        scope_0.push(Box::new(VariableContent::new(name.clone())));
    }

    vec![vec![scope_0]]
}

/// Build the [`VariableSource`] list that mirrors the registry; grit uses this to report
/// variable bindings after a match. Returns `(variable_sources_scope_0,)`.
pub fn build_variable_sources(compiled: &CompiledPattern) -> Vec<Vec<VariableSource>> {
    let mut scope_0 = Vec::new();
    for reserved in ["$filename", "$absolute_path", "$program", "$match"].iter() {
        scope_0.push(VariableSource::new_global((*reserved).to_string()));
    }
    for name in &compiled.variables {
        scope_0.push(VariableSource::new(name.clone(), DEFAULT_FILE_NAME.to_string()));
    }
    vec![scope_0]
}

/// Locate the node inside the parsed program that corresponds to the user's original
/// snippet: the content the user wrote between the wrapper prefix and suffix.
///
/// Approach: walk the whole AST and pick the **outermost** node whose byte span is
/// fully contained within `[user_start, user_end]`. Containment rather than exact
/// equality because the host parser emits spans with slight tolerances (trailing
/// semicolons, whitespace). If multiple candidate nodes exist we prefer the one that
/// covers the most of the user range, since it's the most informative single node we can
/// match against.
///
/// This is what makes the class-body context actually usable: the user writes
/// `public readonly ^type $^name;`, we wrap it in `class __Grit { ... }`, parse, and
/// this function returns the inner `ClassLikeMember` that sits inside the synthetic
/// class body. Same story for function bodies and assignment RHS contexts.
fn find_user_node<'ast, 'arena>(
    program: &'ast Program<'arena>,
    user_start: u32,
    user_end: u32,
) -> Option<Node<'ast, 'arena>> {
    let mut best: Option<Node<'ast, 'arena>> = None;
    let mut best_width: u32 = 0;
    find_user_node_rec(Node::Program(program), user_start, user_end, &mut best, &mut best_width);
    best
}

fn find_user_node_rec<'ast, 'arena>(
    node: Node<'ast, 'arena>,
    user_start: u32,
    user_end: u32,
    best: &mut Option<Node<'ast, 'arena>>,
    best_width: &mut u32,
) {
    let span = node.span();
    let start = span.start.offset;
    let end = span.end.offset;

    if start >= user_start && end <= user_end && end > start {
        let width = end - start;
        if width > *best_width {
            *best_width = width;
            *best = Some(node);
        }
    }

    if start < user_end && end > user_start {
        node.visit_children(|child| find_user_node_rec(child, user_start, user_end, best, best_width));
    }
}

/// Walk an AST node and produce an equivalent [`Pattern<MagoQueryContext>`] tree.
fn lower<'ast, 'arena>(
    node: &Node<'ast, 'arena>,
    source: &str,
    variables: &mut Vec<String>,
) -> Pattern<MagoQueryContext> {
    if let Some(dots_name) = detect_dots_marker(node) {
        return lower_named_dots(dots_name, variables);
    }

    if let Some(name) = detect_meta_name(node) {
        let slot = intern_var(variables, name);
        return Pattern::Variable(Variable::new(0, RESERVED_SLOT_COUNT + slot));
    }

    let kind = node.kind();
    let children: Vec<Node<'ast, 'arena>> = node.children();

    if children.is_empty() {
        let span = node.span();
        let start = span.start.offset as usize;
        let end = span.end.offset as usize;
        let text = if end <= source.len() && start <= end { &source[start..end] } else { "" };
        return Pattern::AstLeafNode(MagoLeafNodePattern::new(kind, text));
    }

    let mut child_patterns = Vec::with_capacity(children.len());
    for child in &children {
        let lowered = lower(child, source, variables);
        child_patterns.push(surface_dots(lowered));
    }

    Pattern::AstNode(Box::new(MagoNodePattern::new(kind, child_patterns)))
}

/// Builds the lowered form of a named-dots marker (`^...name`).
///
/// Anonymous / `^..._` produces a bare [`Pattern::Dots`]. A named marker produces the
/// two-element conjunction `And([Dots, Variable(slot)])`, which the parent
/// [`MagoNodePattern::execute`] recognises: the `Dots` drives sequence absorption and the
/// `Variable` receives the absorbed `NodeList` binding so the RHS can splice it back.
fn lower_named_dots(name: &str, variables: &mut Vec<String>) -> Pattern<MagoQueryContext> {
    if name.is_empty() || name == "_" {
        return Pattern::Dots;
    }
    let slot = intern_var(variables, name);
    Pattern::And(Box::new(And::new(vec![
        Pattern::Dots,
        Pattern::Variable(Variable::new(0, RESERVED_SLOT_COUNT + slot)),
    ])))
}

/// If `p` is a chain of single-child AstNode wrappers around a dots-shaped pattern
/// (`Pattern::Dots` or `Pattern::And([Dots, Variable])`), strip the wrappers and return
/// just the dots shape. Otherwise return `p` unchanged.
fn surface_dots(p: Pattern<MagoQueryContext>) -> Pattern<MagoQueryContext> {
    match p {
        Pattern::Dots => Pattern::Dots,
        Pattern::And(and) if is_named_dots_and(&and) => Pattern::And(and),
        Pattern::AstNode(n) if n.children.len() == 1 => {
            let MagoNodePattern { kind, mut children } = *n;
            let only = children.pop().expect("len == 1");
            let inner = surface_dots(only);
            if matches!(inner, Pattern::Dots) || matches!(&inner, Pattern::And(a) if is_named_dots_and(a)) {
                inner
            } else {
                Pattern::AstNode(Box::new(MagoNodePattern::new(kind, vec![inner])))
            }
        }
        other => other,
    }
}

/// If `node` is an identifier / variable whose name is `µ` + ident, returns the bare
/// variable name (without the `µ`).
fn detect_meta_name<'a>(node: &Node<'a, 'a>) -> Option<&'a str> {
    if detect_dots_marker(node).is_some() {
        return None;
    }
    match node {
        Node::LocalIdentifier(id) => id.value.strip_prefix('µ'),
        Node::Identifier(Identifier::Local(id)) => id.value.strip_prefix('µ'),
        Node::Expression(Expression::Identifier(Identifier::Local(id))) => id.value.strip_prefix('µ'),
        Node::Expression(Expression::ConstantAccess(access)) => match access.name {
            Identifier::Local(LocalIdentifier { value, .. }) => value.strip_prefix('µ'),
            _ => None,
        },
        Node::ConstantAccess(access) => match access.name {
            Identifier::Local(LocalIdentifier { value, .. }) => value.strip_prefix('µ'),
            _ => None,
        },
        Node::Expression(Expression::Variable(PhpVariable::Direct(v))) => v.name.strip_prefix("$µ"),
        Node::Variable(PhpVariable::Direct(v)) => v.name.strip_prefix("$µ"),
        Node::DirectVariable(v) => v.name.strip_prefix("$µ"),
        _ => None,
    }
}

/// Returns `true` if `and` is the two-element conjunction we emit for named dots:
/// `And([Pattern::Dots, Pattern::Variable(_)])`. The [`MagoNodePattern::execute`] loop
/// checks for this shape to bind the absorbed sequence to the variable.
pub(crate) fn is_named_dots_and(and: &And<MagoQueryContext>) -> bool {
    and.patterns.len() == 2
        && matches!(and.patterns[0], Pattern::Dots)
        && matches!(and.patterns[1], Pattern::Variable(_))
}

/// Returns `Some(name)` if this node encodes the sequence-metavariable marker
/// (`__MAGO_DOTS_name`). Produced by [`substitute_metavariable_prefix`] for source
/// tokens of the form `^...name`.
fn detect_dots_marker<'a>(node: &Node<'a, 'a>) -> Option<&'a str> {
    const DOTS: &str = "__MAGO_DOTS_";
    match node {
        Node::LocalIdentifier(id) => id.value.strip_prefix(DOTS),
        Node::Identifier(Identifier::Local(id)) => id.value.strip_prefix(DOTS),
        Node::Expression(Expression::Identifier(Identifier::Local(id))) => id.value.strip_prefix(DOTS),
        Node::Expression(Expression::ConstantAccess(access)) => match access.name {
            Identifier::Local(LocalIdentifier { value, .. }) => value.strip_prefix(DOTS),
            _ => None,
        },
        Node::ConstantAccess(access) => match access.name {
            Identifier::Local(LocalIdentifier { value, .. }) => value.strip_prefix(DOTS),
            _ => None,
        },
        _ => None,
    }
}

/// Interns a metavariable name and returns its slot index (relative to the first
/// user-defined slot, i.e. not including the reserved slots at the front of scope 0).
fn intern_var(variables: &mut Vec<String>, name: &str) -> usize {
    if let Some(pos) = variables.iter().position(|v| v == name) {
        return pos;
    }
    let pos = variables.len();
    variables.push(name.to_string());
    pos
}

/// Convenience accessor: exposes the file id used by the default snippet file. Kept here
/// so downstream CLI code can spin up an ephemeral `File` identical to what the compiler
/// used.
pub fn pattern_file_id() -> FileId {
    FileId::zero()
}
