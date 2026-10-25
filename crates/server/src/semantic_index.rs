//! Typed names and local bindings copied from one transient parse tree.

use mago_span::Span;

use crate::Range;

mod builder;
mod callables;
mod scope;

pub(crate) use builder::build;
pub use callables::CallableReceiver;
pub use callables::CallableReference;

pub use scope::Scope;
pub use scope::ScopeKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    ClassLike,
    Function,
    Constant,
    Namespace,
    Variable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolKey {
    pub kind: SymbolKind,
    pub name: Box<[u8]>,
    /// Local scope ID within this file. Global names have no scope ID.
    pub scope: Option<u32>,
    pub binding: Option<BindingKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BindingKind {
    Local,
    Parameter,
    PromotedProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Occurrence {
    pub range: Range,
    /// The name alone, without a namespace prefix or `$`.
    pub selection_range: Range,
    pub symbol: u32,
    pub declaration: bool,
    /// Import targets and uses of imported names need alias-aware rename handling.
    pub imported: bool,
    pub import: Option<u32>,
    /// The first name in an imported use, such as `B` in `B\Thing`.
    pub alias_range: Option<Range>,
}

#[derive(Debug)]
pub struct ImportBinding {
    pub target_range: Range,
    pub alias_range: Option<Range>,
    pub alias: Box<[u8]>,
    pub kind: SymbolKind,
    pub namespace_range: Range,
    pub target: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectorKind {
    Method,
    Property,
    Constant,
    TraitAdaptation,
    TraitAlias,
}

#[derive(Debug, Clone, Copy)]
pub struct MemberSelector {
    pub range: Range,
    pub receiver: Option<Range>,
    /// Start of the owning class, interface, trait, enum, or anonymous class.
    pub owner: Option<u32>,
    pub kind: SelectorKind,
    pub dynamic: bool,
}

/// A construct that uses local variables by name at runtime, such as `compact('name')`,
/// `extract()`, `get_defined_vars()` or `$$name`.
#[derive(Debug)]
pub struct RuntimeVariables {
    pub range: Range,
    pub scope: u32,
    /// The variable it names, or `None` when it may use any variable in the scope.
    pub name: Option<Box<[u8]>>,
}

#[derive(Debug)]
pub struct SemanticIndex {
    pub symbols: Box<[SymbolKey]>,
    pub occurrences: Box<[Occurrence]>,
    pub scopes: Box<[Scope]>,
    pub imports: Box<[ImportBinding]>,
    pub member_selectors: Box<[MemberSelector]>,
    pub callable_references: Box<[CallableReference]>,
    /// Possible quoted class receivers in callable arrays.
    pub callable_class_literals: Box<[Range]>,
    pub runtime_variables: Box<[RuntimeVariables]>,
}

impl SemanticIndex {
    #[must_use]
    pub fn at_offset(&self, offset: u32) -> Option<&Occurrence> {
        let index = self.occurrences.partition_point(|entry| entry.range.start <= offset).checked_sub(1)?;
        self.occurrences.get(index).filter(|entry| entry.range.contains(offset))
    }

    #[must_use]
    pub fn scope_at(&self, offset: u32) -> Option<&Scope> {
        self.scopes.iter().rev().find(|scope| scope.range.contains(offset))
    }
}

fn range(span: Span) -> Range {
    Range::new(span.start.offset, span.end.offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mago_allocator::LocalArena;
    use mago_database::file::File;
    use mago_names::resolver::NameResolver;
    use mago_syntax::parser::parse_file_borrowed_with_settings;
    use mago_syntax::settings::ParserSettings;
    use std::borrow::Cow;

    fn index(source: &[u8]) -> SemanticIndex {
        let file = File::ephemeral(Cow::Borrowed(b"index.php"), Cow::Owned(source.to_vec()));
        let arena = LocalArena::new();
        let program = parse_file_borrowed_with_settings(&arena, &file, ParserSettings::default());
        assert!(!program.has_errors());
        build(program, &NameResolver::new(&arena).resolve(program), source)
    }

    #[test]
    fn locals_follow_scopes_captures_and_parameter_shadowing() {
        let source = b"<?php function a($x) { $f = function() use (&$x) { return $x; }; $g = fn() => $x; $h = fn($x) => $x; return $x; } function b($x) { return $x; }";
        let index = index(source);
        let xs = index
            .occurrences
            .iter()
            .filter(|entry| index.symbols[entry.symbol as usize].name.as_ref() == b"x")
            .collect::<Vec<_>>();
        let ids = xs.iter().map(|entry| entry.symbol).collect::<Vec<_>>();
        assert_eq!(ids.len(), 9);
        assert_eq!(&ids[..4], &[ids[0]; 4]);
        assert_eq!(ids[4], ids[5]);
        assert_eq!(ids[6], ids[0]);
        assert_eq!(ids[7], ids[8]);
        assert_ne!(ids[0], ids[4]);
        assert_ne!(ids[0], ids[7]);
        assert_eq!(xs.iter().filter(|entry| entry.declaration).count(), 3);
        for entry in xs {
            assert_eq!(&source[entry.selection_range.start as usize..entry.selection_range.end as usize], b"x");
        }
    }

    #[test]
    fn global_names_keep_kind_case_and_exact_spans() {
        let source = br"<?php namespace App; use Lib\{Thing as Alias, function work as go, const FLAG}; class Same {} function Same() {} const Same = 1; const SAME = 2; new sAmE(); SAME(); echo Same, SAME; new Alias(); go(); echo FLAG;";
        let index = index(source);
        let declarations = index
            .occurrences
            .iter()
            .filter(|entry| entry.declaration)
            .map(|entry| &index.symbols[entry.symbol as usize])
            .collect::<Vec<_>>();
        assert!(declarations.iter().any(|key| key.kind == SymbolKind::ClassLike && key.name.as_ref() == br"app\same"));
        assert!(declarations.iter().any(|key| key.kind == SymbolKind::Function && key.name.as_ref() == br"app\same"));
        assert!(declarations.iter().any(|key| key.kind == SymbolKind::Constant && key.name.as_ref() == br"app\Same"));
        assert!(declarations.iter().any(|key| key.kind == SymbolKind::Constant && key.name.as_ref() == br"app\SAME"));
        for (kind, name) in [
            (SymbolKind::ClassLike, br"lib\thing".as_slice()),
            (SymbolKind::Function, br"lib\work".as_slice()),
            (SymbolKind::Constant, br"lib\FLAG".as_slice()),
        ] {
            assert_eq!(
                index
                    .occurrences
                    .iter()
                    .filter(|entry| {
                        let key = &index.symbols[entry.symbol as usize];
                        key.kind == kind && key.name.as_ref() == name && entry.imported
                    })
                    .count(),
                2
            );
        }
        for entry in &index.occurrences {
            assert_eq!(index.at_offset(entry.range.start), Some(entry));
            assert_eq!(index.at_offset(entry.range.end - 1), Some(entry));
            assert_ne!(index.at_offset(entry.range.end), Some(entry));
        }
    }

    #[test]
    fn globals_link_explicit_bindings_and_properties_are_not_locals() {
        let source = b"<?php $x = 1; function f() { global $x; return $x; } class C { public static $x; function m($x) { return C::$x + $x; } }";
        let index = index(source);
        let variables = index
            .occurrences
            .iter()
            .filter(|entry| index.symbols[entry.symbol as usize].kind == SymbolKind::Variable)
            .collect::<Vec<_>>();
        assert_eq!(variables.len(), 5);
        assert_eq!(variables[0].symbol, variables[1].symbol);
        assert_eq!(variables[0].symbol, variables[2].symbol);
        assert_eq!(variables[3].symbol, variables[4].symbol);
        assert_ne!(variables[0].symbol, variables[3].symbol);
        assert_eq!(index.scope_at(variables[4].range.start).map(|scope| scope.kind), Some(ScopeKind::Method));
    }

    #[test]
    fn runtime_variable_uses_and_braced_string_names_are_recorded() {
        let source = br#"<?php function f($row, $name) { extract($row); $total = 1; echo "${total} ${total[0]}"; return compact('total', ['other', $name]) + get_defined_vars() + [$$name, ${'named'}, ${$name}, C::$$name, \compact('x'), Lib\compact('y'), eval('return 1;')]; }"#;
        let index = index(source);
        let text = |range: Range| &source[range.start as usize..range.end as usize];
        let call = b"compact('total', ['other', $name])".as_slice();
        assert_eq!(
            index.runtime_variables.iter().map(|entry| (text(entry.range), entry.name.as_deref())).collect::<Vec<_>>(),
            [
                (b"extract($row)".as_slice(), None),
                (call, Some(b"total".as_slice())),
                (call, Some(b"other".as_slice())),
                (call, None),
                (b"get_defined_vars()", None),
                (b"$$name", None),
                (b"${'named'}", Some(b"named".as_slice())),
                (b"${$name}", None),
                (br"\compact('x')", Some(b"x".as_slice())),
                (b"eval('return 1;')", None),
            ]
        );
        let row = index.occurrences.iter().find(|entry| text(entry.range) == b"$row");
        let scope = row.and_then(|row| index.symbols[row.symbol as usize].scope);
        assert!(scope.is_some() && index.runtime_variables.iter().all(|entry| Some(entry.scope) == scope));
        let totals: Vec<_> = index
            .occurrences
            .iter()
            .filter(|entry| index.symbols[entry.symbol as usize].name.as_ref() == b"total")
            .collect();
        assert_eq!(totals.len(), 3);
        assert!(totals.iter().all(|entry| entry.symbol == totals[0].symbol && text(entry.selection_range) == b"total"));
    }

    #[test]
    fn closure_locals_do_not_capture_without_use() {
        let source = b"<?php $x = 1; $f = function() { $x = 2; return $x; }; echo $x;";
        let index = index(source);
        let xs = index
            .occurrences
            .iter()
            .filter(|entry| index.symbols[entry.symbol as usize].name.as_ref() == b"x")
            .collect::<Vec<_>>();
        assert_eq!(xs.len(), 4);
        assert_eq!(xs[0].symbol, xs[3].symbol);
        assert_eq!(xs[1].symbol, xs[2].symbol);
        assert_ne!(xs[0].symbol, xs[1].symbol);
        assert!(xs[0].declaration && xs[1].declaration);
    }

    #[test]
    fn member_selectors_keep_exact_spans_dynamic_names_and_class_owners() {
        let source = b"<?php trait T { function run() {} } class C { use T { run as alias; T::run insteadof Other; } function f($object, $method) { $object?->run(); $object->$method(); C::$field; C::FLAG; $object->field; $object->run(...); C::run(...); new class { function f() { $this->inner(); } }; $this->outer(); } } $object->outside();";
        let index = index(source);
        let text = |selector: &MemberSelector| &source[selector.range.start as usize..selector.range.end as usize];
        let selectors = &index.member_selectors;
        assert_eq!(selectors.iter().filter(|entry| entry.kind == SelectorKind::TraitAdaptation).count(), 2);
        assert_eq!(selectors.iter().filter(|entry| entry.kind == SelectorKind::TraitAlias).count(), 1);
        assert_eq!(
            selectors.iter().filter(|entry| entry.dynamic).map(text).collect::<Vec<_>>(),
            [b"$method".as_slice()]
        );
        assert!(
            selectors
                .iter()
                .any(|entry| entry.kind == SelectorKind::Property && text(entry) == b"$field" && !entry.dynamic)
        );
        assert!(selectors.iter().any(|entry| entry.kind == SelectorKind::Constant && text(entry) == b"FLAG"));
        let owner = selectors.iter().find(|entry| text(entry) == b"outer").and_then(|entry| entry.owner);
        assert!(owner.is_some());
        assert!(
            selectors
                .iter()
                .filter(|entry| matches!(entry.kind, SelectorKind::TraitAdaptation | SelectorKind::TraitAlias))
                .all(|entry| entry.owner == owner)
        );
        let nested_owner = selectors.iter().find(|entry| text(entry) == b"inner").and_then(|entry| entry.owner);
        assert!(nested_owner.is_some() && nested_owner != owner);
        assert!(selectors.last().is_some_and(|entry| entry.owner.is_none() && text(entry) == b"outside"));
        for entry in selectors
            .iter()
            .filter(|entry| !matches!(entry.kind, SelectorKind::TraitAdaptation | SelectorKind::TraitAlias))
        {
            assert!(entry.receiver.is_some_and(|receiver| receiver.end <= entry.range.start));
        }
    }

    #[test]
    fn callable_class_literals_keep_possible_last_receiver_values() {
        let source = b"<?php $a = ['C', 'run', 0 => 'Other']; $b = [$key => 'Possible', 1 => 'run']; $c = ['Dropped', 'run', ...$more, 0 => 'Kept']; $d = ['Dynamic', $method];";
        let index = index(source);
        let literals: Vec<_> = index
            .callable_class_literals
            .iter()
            .map(|range| &source[range.start as usize..range.end as usize])
            .collect();
        assert_eq!(literals, [b"'Other'".as_slice(), b"'Possible'", b"'Kept'", b"'Dynamic'"]);
    }

    #[test]
    fn callable_arrays_fold_known_keys_and_keep_possible_unknown_methods() {
        let source = b"<?php $a = [0.9 => C::class, 1.8 => 'run']; $b = [0 => C::class, 1 => 'old', 1 => 'run']; $c = [$key => C::class, 1 => 'run']; $d = [C::class, ...['run']]; $e = [C::class, 'old', ...$items, 1 => 'skip'];";
        let index = index(source);
        let calls = &index.callable_references;
        assert_eq!(calls.len(), 5);
        assert_eq!(calls.iter().filter(|call| matches!(call.receiver, Some(CallableReceiver::Unknown))).count(), 3);
        assert!(calls.iter().all(|call| &source[call.method.start as usize..call.method.end as usize] != b"'old'"));
    }

    #[test]
    fn callable_references_keep_quoted_literals_and_receiver_ranges() {
        let source = br#"<?php class C { function f($object) { $a = [$object, 'run']; $b = array(C::class, "ru\x6e"); $c = [0 => 'C', 1 => 'run']; $d = 'C::run'; $e = ['1' => 'run', '0' => C::class]; $data = [1, 'not a method']; $invalid = [1 => 'C', 'run']; } }"#;
        let index = index(source);
        let calls = &index.callable_references;
        assert_eq!(calls.len(), 5);
        assert!(calls.iter().all(|call| call.owner.is_some()));
        assert!(calls.iter().any(|call| matches!(call.receiver, Some(CallableReceiver::Expression(_)))));
        assert!(calls.iter().any(|call| matches!(call.receiver, Some(CallableReceiver::ClassName(_)))));
        assert!(calls.iter().any(|call| matches!(call.receiver, Some(CallableReceiver::Literal(_)))));
        assert!(calls.iter().any(|call| call.receiver.is_none()
            && &source[call.method.start as usize..call.method.end as usize] == b"'C::run'"));
        assert!(
            calls.iter().any(|call| &source[call.method.start as usize..call.method.end as usize] == br#""ru\x6e""#)
        );
    }
}
