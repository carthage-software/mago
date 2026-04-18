//! GritQL-compatible pattern-matching query engine for Mago's PHP AST.
//!
//! This crate plugs Mago's AST into the parser-agnostic `grit-pattern-matcher` engine.
//! The parser is untouched: metavariables in pattern sources are substituted to plain
//! identifiers (`^foo` → `µfoo`) before parsing, and the engine recognizes those
//! identifiers as metavariable holes through [`language::MagoLanguage::is_metavariable`].
//!
//! # Status
//!
//! Experimental. The surface grammar, match path, and rewrite path are implemented and
//! exercised by a 119-case test suite. Output shapes, syntax, and behaviour may change
//! without warning between releases.

pub mod binding;
pub mod code_snippet;
pub mod compiler;
pub mod context;
pub mod file;
pub mod language;
pub mod node;
pub mod node_pattern;
pub mod query;
pub mod query_context;
pub mod resolved_pattern;
pub mod surface;
pub mod testing;
pub mod tree;

pub use compiler::CompileError;
pub use compiler::CompiledPattern;
pub use compiler::compile;
pub use query::Match;
pub use query::QueryError;
pub use query::Rewrite;
pub use query::query;

pub use binding::MagoBinding;
pub use code_snippet::MagoCodeSnippet;
pub use context::MagoExecContext;
pub use file::MagoFile;
pub use language::MagoLanguage;
pub use node::MagoIndex;
pub use node::MagoNode;
pub use node_pattern::MagoLeafNodePattern;
pub use node_pattern::MagoNodePattern;
pub use query_context::MagoQueryContext;
pub use resolved_pattern::MagoResolvedPattern;
pub use tree::MagoTree;

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::path::Path;

    use bumpalo::Bump;
    use mago_database::file::File;
    use mago_syntax::parser::parse_file;

    use super::*;

    fn run(pattern_src: &str, target_src: &str) -> Vec<Match> {
        let pattern_arena = Bump::new();
        let compiled = compile(&pattern_arena, pattern_src).expect("pattern compiles");

        let target_arena = Bump::new();
        let file = File::ephemeral(Cow::Borrowed("<test>"), Cow::Owned(target_src.to_string()));
        let program = parse_file(&target_arena, &file);
        let source = target_arena.alloc_str(file.contents.as_ref());
        let index = MagoIndex::new(program, source);
        query(&compiled, &index, Path::new("<test>")).expect("pattern query runs")
    }

    #[test]
    fn matches_function_call_with_two_metavars() {
        let target = "<?php str_contains($a, $b); str_contains('foo', 'bar');";
        let matches = run("str_contains(^a, ^b)", target);
        assert_eq!(matches.len(), 2, "expected two matches, got {:?}", matches);
    }

    #[test]
    fn respects_literal_argument_in_pattern() {
        let target = "<?php str_contains($a, 'bar'); str_contains($c, 'baz');";
        let matches = run("str_contains(^x, 'bar')", target);
        assert_eq!(matches.len(), 1, "expected 1 match, got {:?}", matches);
    }

    #[test]
    fn captures_metavariable_value() {
        let target = "<?php eval($dangerous);";
        let matches = run("eval(^x)", target);
        assert_eq!(matches.len(), 1);
        let captures = &matches[0].captures;
        assert_eq!(captures.len(), 1);
        assert_eq!(captures[0].0, "x");
        assert!(captures[0].1.contains("dangerous"), "captured text: {:?}", captures[0].1);
    }

    #[test]
    fn same_variable_must_bind_consistently() {
        // `^x` appears twice in the pattern; both occurrences must bind to the same
        // target text.
        let target = "<?php strcmp($a, $a); strcmp($a, $b);";
        let matches = run("strcmp(^x, ^x)", target);
        assert_eq!(matches.len(), 1, "expected only the matching call, got {:?}", matches);
    }

    #[test]
    fn empty_pattern_rejected() {
        let pattern_arena = Bump::new();
        assert!(matches!(compile(&pattern_arena, "   "), Err(CompileError::Empty)));
    }
}
