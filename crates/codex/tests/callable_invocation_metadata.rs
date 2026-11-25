#![allow(clippy::unwrap_used)]

use std::borrow::Cow;
use std::mem::size_of;

use mago_allocator::LocalArena;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::metadata::function_like::FunctionLikeKind;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::metadata::parameter::CallableInvocationKind;
use mago_codex::metadata::parameter::FunctionLikeParameterMetadata;
use mago_codex::misc::VariableIdentifier;
use mago_codex::scanner::scan_program;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_php_version::PHPVersion;
use mago_span::Span;
use mago_syntax::parser::parse_file;
use mago_word::word;

fn scan(source: &str) -> CodebaseMetadata {
    let arena = LocalArena::new();
    let file = File::ephemeral(Cow::Borrowed(b"callable-timing.php"), Cow::Owned(source.as_bytes().to_vec()));
    let program = parse_file(&arena, &file);
    assert!(!program.has_errors(), "{:?}", program.errors);
    let names = NameResolver::new(&arena).resolve(program);

    scan_program(&arena, &file, program, &names, PHPVersion::LATEST)
}

#[test]
fn scans_immediate_and_later_callable_parameters_in_functions_and_arrays() {
    let mut codebase = scan(
        "<?php\n\
         /** @param-immediately-invoked-callable $callback */\n\
         function immediate(callable $callback): void {}\n\
         /**\n\
          * @param array<string, callable> $callbacks\n\
          * @param-later-invoked-callable $callbacks\n\
          */\n\
         function later(array $callbacks): void {}\n",
    );

    assert_eq!(
        codebase
            .get_function(b"immediate")
            .unwrap()
            .get_parameter(word("$callback"))
            .unwrap()
            .callable_invocation_kind(),
        Some(CallableInvocationKind::Immediately)
    );
    assert_eq!(
        codebase.get_function(b"later").unwrap().get_parameter(word("$callbacks")).unwrap().callable_invocation_kind(),
        Some(CallableInvocationKind::Later)
    );
    assert!(codebase.take_issues(true).is_empty());
}

#[test]
fn rejects_unknown_non_callable_and_conflicting_timing_tags() {
    let mut codebase = scan(
        "<?php\n\
         /** @param-immediately-invoked-callable $missing */\n\
         function unknown(callable $callback): void {}\n\
         /** @param-later-invoked-callable $value */\n\
         function non_callable(int $value): void {}\n\
         /**\n\
          * @param-immediately-invoked-callable $callback\n\
          * @param-later-invoked-callable $callback\n\
          */\n\
         function conflicting(callable $callback): void {}\n",
    );

    let issues = codebase.take_issues(true);
    assert_eq!(issues.len(), 3, "{issues:#?}");
    assert!(issues.iter().all(|issue| issue.code.as_deref() == Some("invalid-param-tag")));
    assert_eq!(
        codebase
            .get_function(b"conflicting")
            .unwrap()
            .get_parameter(word("$callback"))
            .unwrap()
            .callable_invocation_kind(),
        None
    );
}

#[test]
fn patch_merge_copies_only_an_explicit_callable_timing_tag() {
    let span = Span::dummy(0, 0);
    let flags = MetadataFlags::empty();
    let parameter = || FunctionLikeParameterMetadata::new(VariableIdentifier(word("$callback")), span, span, flags);
    let mut original = FunctionLikeMetadata::new(FunctionLikeKind::Function, word("f"), word("f"), span, flags)
        .with_parameters([parameter()]);
    let mut patch = FunctionLikeMetadata::new(FunctionLikeKind::Function, word("f"), word("f"), span, flags)
        .with_parameters([parameter()]);
    patch.parameters[0].set_callable_invocation_kind(Some(CallableInvocationKind::Immediately));

    original.apply_patch(&mut patch);
    assert_eq!(original.parameters[0].callable_invocation_kind(), Some(CallableInvocationKind::Immediately));

    let mut empty_patch = FunctionLikeMetadata::new(FunctionLikeKind::Function, word("f"), word("f"), span, flags)
        .with_parameters([parameter()]);
    original.apply_patch(&mut empty_patch);
    assert_eq!(original.parameters[0].callable_invocation_kind(), Some(CallableInvocationKind::Immediately));
}

#[test]
fn callable_timing_metadata_size_is_compact() {
    assert_eq!(size_of::<Option<CallableInvocationKind>>(), 1);
    assert!(size_of::<FunctionLikeParameterMetadata>() <= 384);
}

#[cfg(feature = "serde")]
#[test]
fn serde_roundtrip_and_missing_field_default_preserve_callable_timing() {
    let span = Span::dummy(0, 0);
    let mut parameter =
        FunctionLikeParameterMetadata::new(VariableIdentifier(word("$callback")), span, span, MetadataFlags::empty());
    parameter.set_callable_invocation_kind(Some(CallableInvocationKind::Later));

    let serialized = serde_json::to_value(&parameter).unwrap();
    let roundtrip: FunctionLikeParameterMetadata = serde_json::from_value(serialized.clone()).unwrap();
    assert_eq!(roundtrip.callable_invocation_kind(), Some(CallableInvocationKind::Later));

    let mut legacy = serialized;
    legacy["flags"] = serde_json::to_value(MetadataFlags::empty()).unwrap();
    let restored: FunctionLikeParameterMetadata = serde_json::from_value(legacy).unwrap();
    assert_eq!(restored.callable_invocation_kind(), None);
}
