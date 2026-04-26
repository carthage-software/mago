mod combiner_common;

use combiner_common::*;

use mago_codex::ttype::atomic::TAtomic;

#[test]
fn idempotent_zoo() {
    for atom in string_atom_zoo() {
        for n in 1..=8 {
            assert_self_idempotent(atom.clone(), n);
        }
    }
}

#[test]
fn singleton_passthrough() {
    for atom in string_atom_zoo() {
        let r = combine_default(vec![atom.clone()]);
        assert_eq!(r.len(), 1);
        assert_eq!(atomic_id_string(&r[0]), atomic_id_string(&atom));
    }
}

#[test]
fn duplicate_literal_strings_collapse() {
    for s in ["", "hello", "0", "foo bar", "123", "Hello World"] {
        for n in 1..=10 {
            assert_combines_to(vec![t_lit_string(s); n], vec![t_lit_string(s)]);
        }
    }
}

#[test]
fn string_absorbs_literal_either_order() {
    for s in ["", "hi", "0", "Hello", "123", " "] {
        assert_combines_to(vec![t_string(), t_lit_string(s)], vec![t_string()]);
        assert_combines_to(vec![t_lit_string(s), t_string()], vec![t_string()]);
    }
}

#[test]
fn non_empty_absorbs_compatible_literal() {
    for s in ["a", "hi", "0", "Hello"] {
        assert_combines_to(vec![t_non_empty_string(), t_lit_string(s)], vec![t_non_empty_string()]);
        assert_combines_to(vec![t_lit_string(s), t_non_empty_string()], vec![t_non_empty_string()]);
    }
}

#[test]
fn non_empty_first_keeps_empty_literal_separate() {
    let result = combine_default(vec![t_non_empty_string(), t_lit_string("")]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["non-empty-string", "string('')"]);
}

#[test]
fn empty_literal_first_downgrades_non_empty_to_general_string() {
    let result = combine_default(vec![t_lit_string(""), t_non_empty_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["string"]);
}

#[test]
fn numeric_string_absorbs_numeric_literal() {
    for s in ["0", "1", "-1", "123", "1.5", "1e10", "-0", "0.0"] {
        assert_combines_to(vec![t_numeric_string(), t_lit_string(s)], vec![t_numeric_string()]);
    }
}

#[test]
fn numeric_string_first_keeps_non_numeric_literal_separate() {
    for s in ["hi", "abc", "12abc", "abc123"] {
        let result = combine_default(vec![t_numeric_string(), t_lit_string(s)]);
        let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
        ids.sort();
        assert_eq!(ids.len(), 2, "numeric ∨ '{s}'");
        assert!(ids.contains(&"numeric-string".to_string()));
    }
}

#[test]
fn non_numeric_literal_first_with_numeric_string_keeps_separate() {
    for s in ["hi", "abc"] {
        let result = combine_default(vec![t_lit_string(s), t_numeric_string()]);
        let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
        ids.sort();
        assert!(ids.contains(&"numeric-string".to_string()));
    }
}

#[test]
fn lowercase_string_absorbs_lowercase_literal() {
    for s in ["hi", "abc", "hello"] {
        assert_combines_to(vec![t_lower_string(), t_lit_string(s)], vec![t_lower_string()]);
    }
}

#[test]
fn lowercase_with_empty_literal_collapses_to_general_string() {
    let result = combine_default(vec![t_lower_string(), t_lit_string("")]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "string");
}

#[test]
fn lowercase_string_first_keeps_uppercase_literal_separate() {
    for s in ["HI", "ABC", "Hello"] {
        let result = combine_default(vec![t_lower_string(), t_lit_string(s)]);
        let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
        ids.sort();
        assert!(ids.contains(&"lowercase-string".to_string()));
        assert_eq!(ids.len(), 2);
    }
}

#[test]
fn uppercase_string_absorbs_uppercase_literal() {
    for s in ["HI", "ABC", "FOO"] {
        assert_combines_to(vec![t_upper_string(), t_lit_string(s)], vec![t_upper_string()]);
    }
}

#[test]
fn uppercase_with_empty_literal_collapses_to_general_string() {
    let result = combine_default(vec![t_upper_string(), t_lit_string("")]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "string");
}

#[test]
fn uppercase_string_first_keeps_lowercase_literal_separate() {
    for s in ["hi", "abc", "Hello"] {
        let result = combine_default(vec![t_upper_string(), t_lit_string(s)]);
        let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
        ids.sort();
        assert!(ids.contains(&"uppercase-string".to_string()));
        assert_eq!(ids.len(), 2);
    }
}

#[test]
fn truthy_string_absorbs_truthy_literal() {
    for s in ["hi", "abc", "1", "true", "Hello"] {
        assert_combines_to(vec![t_truthy_string(), t_lit_string(s)], vec![t_truthy_string()]);
    }
}

#[test]
fn truthy_string_first_keeps_falsy_literal_separate() {
    for s in ["", "0"] {
        let result = combine_default(vec![t_truthy_string(), t_lit_string(s)]);
        let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
        ids.sort();
        assert!(ids.contains(&"truthy-string".to_string()));
        assert!(ids.iter().any(|s| s.starts_with("string(")));
    }
}

#[test]
fn lower_or_upper_collapses_to_general_string() {
    assert_combines_to(vec![t_lower_string(), t_upper_string()], vec![t_string()]);
    assert_combines_to(vec![t_upper_string(), t_lower_string()], vec![t_string()]);
}

#[test]
fn non_empty_or_lower_collapses_to_general_string() {
    assert_combines_to(vec![t_non_empty_string(), t_lower_string()], vec![t_string()]);
    assert_combines_to(vec![t_lower_string(), t_non_empty_string()], vec![t_string()]);
}

#[test]
fn truthy_or_non_empty_collapses_to_non_empty() {
    assert_combines_to(vec![t_truthy_string(), t_non_empty_string()], vec![t_non_empty_string()]);
    assert_combines_to(vec![t_non_empty_string(), t_truthy_string()], vec![t_non_empty_string()]);
}

#[test]
fn truthy_or_numeric_collapses() {
    let result = combine_default(vec![t_truthy_string(), t_numeric_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids.len(), 1);
}

#[test]
fn lower_or_numeric_keeps_axes() {
    let result = combine_default(vec![t_lower_string(), t_numeric_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], "string");
}

#[test]
fn two_distinct_literals_kept() {
    let pairs = [("a", "b"), ("hi", "hello"), ("0", "1"), ("", "x")];
    for (a, b) in pairs {
        let result = combine_default(vec![t_lit_string(a), t_lit_string(b)]);
        assert_eq!(result.len(), 2, "'{a}' vs '{b}'");
    }
}

#[test]
fn case_sensitive_literals_kept_apart() {
    for (a, b) in [("a", "A"), ("Hello", "hello"), ("XYZ", "xyz")] {
        let result = combine_default(vec![t_lit_string(a), t_lit_string(b)]);
        assert_eq!(result.len(), 2);
    }
}

#[test]
fn n_distinct_literals_kept() {
    for n in [3_usize, 5, 10, 50, 100] {
        let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_string(&format!("s{i}"))).collect();
        let result = combine_default(inputs);
        assert_eq!(result.len(), n);
    }
}

#[test]
fn many_distinct_literals_exceed_threshold_generalise() {
    let n = 200_usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_string(&format!("s{i}"))).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "string");
}

#[test]
fn under_threshold_keeps_literals() {
    let n = 100_usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_string(&format!("s{i}"))).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), n);
}

#[test]
fn custom_low_threshold_generalises_quickly() {
    let inputs: Vec<TAtomic> = (0..20_usize).map(|i| t_lit_string(&format!("s{i}"))).collect();
    let result = combine_with_string_threshold(inputs, 5);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "string");
}

#[test]
fn threshold_zero_immediate_generalisation_when_two_or_more() {
    let inputs = vec![t_lit_string("x"), t_lit_string("y")];
    let result = combine_with_string_threshold(inputs, 0);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "string");
}

#[test]
fn callable_string_with_literal_keeps_truthy_axis() {
    let result = combine_default(vec![t_callable_string(), t_lit_string("foo")]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["truthy-string"]);
}

#[test]
fn unspec_literal_absorbs_specific_literal() {
    let result = combine_default(vec![t_unspec_lit_string(false), t_lit_string("hi")]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], "literal-string");
}

#[test]
fn non_empty_unspec_literal_absorbs_specific() {
    let result = combine_default(vec![t_unspec_lit_string(true), t_lit_string("hi")]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], "non-empty-literal-string");
}

#[test]
fn many_literals_with_general_collapse() {
    let mut inputs = vec![t_string()];
    for i in 0..50 {
        inputs.push(t_lit_string(&format!("s{i}")));
    }
    assert_combines_to(inputs, vec![t_string()]);
}

#[test]
fn many_compatible_literals_with_non_empty_collapse() {
    let mut inputs = vec![t_non_empty_string()];
    for i in 0..30 {
        inputs.push(t_lit_string(&format!("s{i}"))); // all non-empty
    }
    assert_combines_to(inputs, vec![t_non_empty_string()]);
}

#[test]
fn mixed_compatible_and_incompatible_literals_with_non_empty() {
    let inputs = vec![
        t_non_empty_string(),
        t_lit_string("a"),
        t_lit_string("b"),
        t_lit_string(""), // incompatible
        t_lit_string("c"),
    ];
    let result = combine_default(inputs);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert!(ids.contains(&"non-empty-string".to_string()));
    assert!(ids.contains(&"string('')".to_string()));
}

#[test]
fn literal_only_combine_order_independent() {
    let cases: Vec<Vec<TAtomic>> = vec![
        vec![t_lit_string("a"), t_lit_string("b"), t_lit_string("c")],
        vec![t_lit_string("hello"), t_lit_string("world"), t_lit_string("foo")],
        vec![t_lit_string(""), t_lit_string("0"), t_lit_string("x")],
    ];
    for case in cases {
        let r1 = combine_default(case.clone());
        let mut reversed = case;
        reversed.reverse();
        let r2 = combine_default(reversed);
        let mut a: Vec<String> = r1.iter().map(atomic_id_string).collect();
        let mut b: Vec<String> = r2.iter().map(atomic_id_string).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);
    }
}

#[test]
fn lit_lower_with_lit_upper_kept_apart() {
    let result = combine_default(vec![t_lit_string("abc"), t_lit_string("ABC")]);
    assert_eq!(result.len(), 2);
}

#[test]
fn many_lower_literals_with_non_empty_lower_collapse() {
    let inputs: Vec<TAtomic> =
        std::iter::once(t_lower_string()).chain((0..20).map(|i| t_lit_string(&format!("s{i}")))).collect();
    assert_combines_to(inputs, vec![t_lower_string()]);
}

#[test]
fn many_uppercase_literals_with_uppercase_collapse() {
    let inputs: Vec<TAtomic> =
        std::iter::once(t_upper_string()).chain((0..20).map(|i| t_lit_string(&format!("S{i}")))).collect();
    assert_combines_to(inputs, vec![t_upper_string()]);
}

fn string_atom_zoo() -> Vec<TAtomic> {
    vec![
        t_string(),
        t_lit_string(""),
        t_lit_string("hi"),
        t_lit_string("0"),
        t_lit_string("123"),
        t_lit_string("Hello"),
        t_lit_string("HELLO"),
        t_non_empty_string(),
        t_numeric_string(),
        t_lower_string(),
        t_upper_string(),
        t_truthy_string(),
        t_callable_string(),
        t_unspec_lit_string(false),
        t_unspec_lit_string(true),
    ]
}
