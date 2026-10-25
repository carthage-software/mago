use foldhash::HashSet;
use mago_allocator::LocalArena;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::signature_builder::build_file_signature;
use mago_linter::settings::Settings as LinterSettings;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_syntax::parser::parse_file;
use mago_syntax::settings::ParserSettings;
use mago_word::WordSet;

use super::*;
use crate::call_analysis::call_targets;

fn fixture(source: &str) -> (File, CodebaseMetadata, FileAnalysis) {
    let file = File::ephemeral(b"calls.php".as_slice().into(), source.as_bytes().to_vec().into());
    let arena = LocalArena::new();
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors());
    let names = NameResolver::new(&arena).resolve(program);
    let mut codebase = scan_program(&arena, &file, program, &names, PHPVersion::PHP85);
    codebase.set_file_signature(file.id, build_file_signature(&file, program, &names));
    populate_codebase(&mut codebase, &mut SymbolReferences::new(), WordSet::default(), HashSet::default());
    let linter = crate::linter::LinterContext::new(LinterSettings::default(), ParserSettings::default(), false);
    let analysis = crate::file_analysis::build(&file, &linter, false);
    (file, codebase, analysis)
}

#[test]
fn anonymous_constructor_arguments_keep_the_outer_class_scope() {
    let source = "<?php class Outer {
        public static function send(string $outer): string { return $outer; }
        public function make(): object {
            return new CLASS(self::send('argument')) {
                public function __construct(string $value) {}
                public static function send(string $inner): string { return $inner; }
                public function run(): string { return self::send('body'); }
            };
        }
    }";
    let (file, codebase, analysis) = fixture(source);
    for (call, expected) in [("send('argument')", "class Outer"), ("send('body')", "new CLASS")] {
        let Some(offset) = source.find(call) else { panic!("fixture call missing") };
        let Some(class) = crate::call_analysis::lexical_class(&codebase, &file, &analysis, offset as u32) else {
            panic!("lexical class missing")
        };
        assert_eq!(Some(class.span.start.offset as usize), source.find(expected));
    }
}

#[test]
fn calls_use_imports_namespace_fallback_and_inherited_constructors() {
    let (file, codebase, analysis) = fixture(
        r"<?php
namespace { function fallback() {} function qualified() {} }
namespace Lib { function work() {} class Base { function __construct() {} static function run() {} } }
namespace App {
    use function Lib\work as imported;
    use Lib\Base;
    class Child extends Base {}
    function own() { imported(); fallback(); \qualified(); new Child(); Child::run(); }
}",
    );
    let targets: Vec<_> = analysis
        .calls
        .iter()
        .flat_map(|call| call_targets(&codebase, &file, &analysis, None, call).into_iter().map(|key| key.as_string()))
        .collect();
    assert_eq!(targets, ["lib\\work", "fallback", "qualified", "lib\\base::__construct", "lib\\base::run"]);
    for call in &analysis.calls {
        assert_eq!(
            owner_at(&codebase, &file, &analysis, call.expression.start).map(|key| key.as_string()),
            Some("app\\own".into())
        );
    }
}

#[test]
fn callable_references_are_not_calls_and_closures_own_their_calls() {
    let (file, codebase, analysis) = fixture(
        "<?php
function target() {}
class C { static function run() {} }
function outer() {
    $a = target(...); $b = C::run(...);
    $c = function () { target(); };
    $d = fn() => target();
    function inner() { target(); }
    target();
}
target();",
    );
    assert_eq!(analysis.calls.len(), 5);
    let owners: Vec<_> =
        analysis.calls.iter().map(|call| owner_at(&codebase, &file, &analysis, call.expression.start)).collect();
    assert!(owners[0].is_some_and(|key| key.is_closure()));
    assert!(owners[1].is_some_and(|key| key.is_closure()));
    assert_ne!(owners[0], owners[1]);
    assert_eq!(owners[2], Some(FunctionLikeIdentifier::Function(word("inner"))));
    assert_eq!(owners[3], Some(FunctionLikeIdentifier::Function(word("outer"))));
    assert_eq!(owners[4], None);
    for owner in owners.into_iter().flatten() {
        assert!(hierarchy_item(&codebase, owner).is_some());
    }
    let mut builder = GraphBuilder::default();
    collect_file_calls(&codebase, &file, &analysis, None, &mut builder);
    let graph = builder.finish();
    let incoming = graph.incoming(&codebase, FunctionLikeIdentifier::Function(word("target")));
    assert_eq!(incoming.len(), 4);
    assert!(incoming.iter().all(|call| call.ranges.len() == 1));
    let outer = graph.outgoing(&codebase, FunctionLikeIdentifier::Function(word("outer")));
    assert_eq!(outer.len(), 1);
    assert_eq!(outer[0].item.name, "target");
    assert_eq!(outer[0].ranges, [analysis.calls[3].selection]);
}

#[test]
fn graph_groups_each_direction_and_keeps_source_ranges_once() {
    let (_, codebase, _) = fixture("<?php function a() {} function b() {} function c() {}");
    let a = FunctionLikeIdentifier::Function(word("a"));
    let b = FunctionLikeIdentifier::Function(word("b"));
    let c = FunctionLikeIdentifier::Function(word("c"));
    let mut builder = GraphBuilder::default();
    builder.add(c, b, Range::new(40, 41));
    builder.add(a, b, Range::new(20, 21));
    builder.add(a, c, Range::new(30, 31));
    builder.add(a, b, Range::new(10, 11));
    builder.add(a, b, Range::new(20, 21));
    let graph = builder.finish();
    let outgoing = graph.outgoing(&codebase, a);
    assert_eq!(outgoing.iter().map(|call| call.item.name.as_str()).collect::<Vec<_>>(), ["b", "c"]);
    assert_eq!(outgoing[0].ranges, [Range::new(10, 11), Range::new(20, 21)]);
    assert_eq!(outgoing[1].ranges, [Range::new(30, 31)]);
    let incoming = graph.incoming(&codebase, b);
    assert_eq!(incoming.iter().map(|call| call.item.name.as_str()).collect::<Vec<_>>(), ["a", "c"]);
    assert_eq!(incoming[0].ranges, outgoing[0].ranges);
    assert_eq!(incoming[1].ranges, [Range::new(40, 41)]);
    assert!(graph.incoming(&codebase, a).is_empty());
    assert!(graph.incoming(&codebase, FunctionLikeIdentifier::Function(word("absent"))).is_empty());
}

#[test]
fn lexical_this_respects_static_closures_functions_and_anonymous_classes() {
    let (file, codebase, analysis) = fixture(
        "<?php class Outer {
            function target() {}
            function run() {
                $this->target();
                $a = fn() => $this->target();
                $b = static fn() => $this->target();
                $c = function() { $this->target(); };
                $d = static function() { $this->target(); };
                function nested() { $this->target(); }
                $e = new class { function target() {} function run() { $this->target(); } };
            }
            static function invalid() { $this->target(); }
        }",
    );
    let targets: Vec<_> =
        analysis.calls.iter().map(|call| call_targets(&codebase, &file, &analysis, None, call)).collect();
    assert_eq!(targets.iter().map(Vec::len).collect::<Vec<_>>(), [1, 1, 0, 1, 0, 0, 1, 0]);
    assert_ne!(targets[0], targets[6]);
    let mut builder = GraphBuilder::default();
    collect_file_calls(&codebase, &file, &analysis, None, &mut builder);
    let graph = builder.finish();
    let callers = graph.incoming(&codebase, targets[0][0]);
    assert_eq!(callers.len(), 3);
    let anonymous = graph.incoming(&codebase, targets[6][0]);
    assert_eq!(anonymous.len(), 1);
    assert_eq!(anonymous[0].ranges, [analysis.calls[6].selection]);
}

#[test]
fn duplicate_declarations_do_not_borrow_another_files_caller() {
    let source = "<?php function caller() { target(); } function target() {}";
    let (first, mut codebase, _) = fixture(source);
    let second = File::ephemeral(b"duplicate.php".as_slice().into(), source.as_bytes().to_vec().into());
    codebase.set_file_signature(second.id, codebase.file_signatures[&first.id].clone());
    let linter = crate::linter::LinterContext::new(LinterSettings::default(), ParserSettings::default(), false);
    let analysis = crate::file_analysis::build(&second, &linter, false);
    assert_eq!(owner_at(&codebase, &second, &analysis, analysis.calls[0].selection.start), None);
    let mut builder = GraphBuilder::default();
    collect_file_calls(&codebase, &second, &analysis, None, &mut builder);
    assert!(builder.finish().incoming(&codebase, FunctionLikeIdentifier::Function(word("target"))).is_empty());
}
