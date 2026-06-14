mod common;

use common::*;

use mago_oracle::ty::Atom;

fn string_atom_zoo<'arena>(f: &mut Fixture<'_, 'arena>) -> Vec<Atom<'arena>> {
    vec![
        f.t_string(),
        f.t_lit_string(""),
        f.t_lit_string("hi"),
        f.t_lit_string("0"),
        f.t_lit_string("123"),
        f.t_lit_string("Hello"),
        f.t_lit_string("HELLO"),
        f.t_non_empty_string(),
        f.t_numeric_string(),
        f.t_lower_string(),
        f.t_upper_string(),
        f.t_truthy_string(),
        f.t_callable_string(),
        f.t_unspec_lit_string(false),
        f.t_unspec_lit_string(true),
    ]
}

#[test]
fn idempotent_zoo() {
    fixture(|f| {
        for atom in string_atom_zoo(f) {
            for n in 1..=8 {
                assert_self_idempotent(f, atom, n);
            }
        }
    });
}

#[test]
fn singleton_passthrough() {
    fixture(|f| {
        for atom in string_atom_zoo(f) {
            let r = combine_default(f, vec![atom]);
            assert_eq!(r.len(), 1);
            assert_eq!(r[0], atom);
        }
    });
}

#[test]
fn duplicate_literal_strings_collapse() {
    fixture(|f| {
        for s in ["", "hello", "0", "foo bar", "123", "Hello World"] {
            let lit = f.t_lit_string(s);
            for n in 1..=10 {
                assert_combines_to(f, vec![lit; n], vec![lit]);
            }
        }
    });
}

#[test]
fn string_absorbs_literal_either_order() {
    fixture(|f| {
        for s in ["", "hi", "0", "Hello", "123", " "] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_string(), lit], vec![f.t_string()]);
            assert_combines_to(f, vec![lit, f.t_string()], vec![f.t_string()]);
        }
    });
}

#[test]
fn non_empty_absorbs_compatible_literal() {
    fixture(|f| {
        for s in ["a", "hi", "0", "Hello"] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_non_empty_string(), lit], vec![f.t_non_empty_string()]);
            assert_combines_to(f, vec![lit, f.t_non_empty_string()], vec![f.t_non_empty_string()]);
        }
    });
}

#[test]
fn non_empty_first_keeps_empty_literal_separate() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let result = combine_default(f, vec![f.t_non_empty_string(), empty]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&f.t_non_empty_string()));
        assert!(result.contains(&empty));
    });
}

#[test]
fn empty_literal_first_downgrades_non_empty_to_general_string() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        assert_combines_to(f, vec![empty, f.t_non_empty_string()], vec![f.t_string()]);
    });
}

#[test]
fn numeric_string_absorbs_numeric_literal() {
    fixture(|f| {
        for s in ["0", "1", "-1", "123", "1.5", "1e10", "-0", "0.0"] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_numeric_string(), lit], vec![f.t_numeric_string()]);
        }
    });
}

#[test]
fn numeric_string_first_keeps_non_numeric_literal_separate() {
    fixture(|f| {
        for s in ["hi", "abc", "12abc", "abc123"] {
            let lit = f.t_lit_string(s);
            let result = combine_default(f, vec![f.t_numeric_string(), lit]);
            assert_eq!(result.len(), 2, "numeric | '{s}'");
            assert!(result.contains(&f.t_numeric_string()));
        }
    });
}

#[test]
fn non_numeric_literal_first_with_numeric_string_keeps_separate() {
    fixture(|f| {
        for s in ["hi", "abc"] {
            let lit = f.t_lit_string(s);
            let result = combine_default(f, vec![lit, f.t_numeric_string()]);
            assert!(result.contains(&f.t_numeric_string()));
        }
    });
}

#[test]
fn lowercase_string_absorbs_lowercase_literal() {
    fixture(|f| {
        for s in ["hi", "abc", "hello"] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_lower_string(), lit], vec![f.t_lower_string()]);
        }
    });
}

#[test]
fn lowercase_with_empty_literal_collapses_to_general_string() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        assert_combines_to(f, vec![f.t_lower_string(), empty], vec![f.t_lower_string()]);
    });
}

#[test]
fn lowercase_string_first_keeps_uppercase_literal_separate() {
    fixture(|f| {
        for s in ["HI", "ABC", "Hello"] {
            let lit = f.t_lit_string(s);
            let result = combine_default(f, vec![f.t_lower_string(), lit]);
            assert!(result.contains(&f.t_lower_string()));
            assert_eq!(result.len(), 2);
        }
    });
}

#[test]
fn uppercase_string_absorbs_uppercase_literal() {
    fixture(|f| {
        for s in ["HI", "ABC", "FOO"] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_upper_string(), lit], vec![f.t_upper_string()]);
        }
    });
}

#[test]
fn uppercase_with_empty_literal_collapses_to_general_string() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        assert_combines_to(f, vec![f.t_upper_string(), empty], vec![f.t_upper_string()]);
    });
}

#[test]
fn uppercase_string_first_keeps_lowercase_literal_separate() {
    fixture(|f| {
        for s in ["hi", "abc", "Hello"] {
            let lit = f.t_lit_string(s);
            let result = combine_default(f, vec![f.t_upper_string(), lit]);
            assert!(result.contains(&f.t_upper_string()));
            assert_eq!(result.len(), 2);
        }
    });
}

#[test]
fn truthy_string_absorbs_truthy_literal() {
    fixture(|f| {
        for s in ["hi", "abc", "1", "true", "Hello"] {
            let lit = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_truthy_string(), lit], vec![f.t_truthy_string()]);
        }
    });
}

#[test]
fn truthy_string_first_keeps_falsy_literal_separate() {
    fixture(|f| {
        for s in ["", "0"] {
            let lit = f.t_lit_string(s);
            let result = combine_default(f, vec![f.t_truthy_string(), lit]);
            assert!(result.contains(&f.t_truthy_string()));
        }
    });
}

#[test]
fn lower_or_upper_collapses_to_general_string() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_lower_string(), f.t_upper_string()], vec![f.t_string()]);
        assert_combines_to(f, vec![f.t_upper_string(), f.t_lower_string()], vec![f.t_string()]);
    });
}

#[test]
fn non_empty_or_lower_collapses_to_general_string() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_non_empty_string(), f.t_lower_string()], vec![f.t_string()]);
        assert_combines_to(f, vec![f.t_lower_string(), f.t_non_empty_string()], vec![f.t_string()]);
    });
}

#[test]
fn truthy_or_non_empty_collapses_to_non_empty() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_truthy_string(), f.t_non_empty_string()], vec![f.t_non_empty_string()]);
        assert_combines_to(f, vec![f.t_non_empty_string(), f.t_truthy_string()], vec![f.t_non_empty_string()]);
    });
}

#[test]
fn truthy_or_numeric_collapses() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_truthy_string(), f.t_numeric_string()]);
        assert_eq!(result.len(), 1);
    });
}

#[test]
fn lower_or_numeric_keeps_axes() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_lower_string(), f.t_numeric_string()]);
        assert_eq!(result, vec![f.t_string()]);
    });
}

#[test]
fn two_distinct_literals_kept() {
    fixture(|f| {
        let pairs = [("a", "b"), ("hi", "hello"), ("0", "1"), ("", "x")];
        for (a, b) in pairs {
            let inputs = vec![f.t_lit_string(a), f.t_lit_string(b)];
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), 2, "'{a}' vs '{b}'");
        }
    });
}

#[test]
fn case_sensitive_literals_kept_apart() {
    fixture(|f| {
        for (a, b) in [("a", "A"), ("Hello", "hello"), ("XYZ", "xyz")] {
            let inputs = vec![f.t_lit_string(a), f.t_lit_string(b)];
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), 2);
        }
    });
}

#[test]
fn n_distinct_literals_kept() {
    fixture(|f| {
        for n in [3usize, 5, 10, 50, 100] {
            let inputs: Vec<_> = (0..n).map(|i| f.t_lit_string(&format!("s{i}"))).collect();
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), n);
        }
    });
}

#[test]
fn many_distinct_literals_exceed_threshold_generalise() {
    fixture(|f| {
        let n = 200usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_string(&format!("s{i}"))).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_string()]);
    });
}

#[test]
fn under_threshold_keeps_literals() {
    fixture(|f| {
        let n = 100usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_string(&format!("s{i}"))).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), n);
    });
}

#[test]
fn custom_low_threshold_generalises_quickly() {
    fixture(|f| {
        let inputs: Vec<_> = (0..20usize).map(|i| f.t_lit_string(&format!("s{i}"))).collect();
        let result = combine_with_string_threshold(f, inputs, 5);
        assert_eq!(result, vec![f.t_string()]);
    });
}

#[test]
fn threshold_zero_immediate_generalisation_when_two_or_more() {
    fixture(|f| {
        let inputs = vec![f.t_lit_string("x"), f.t_lit_string("y")];
        let result = combine_with_string_threshold(f, inputs, 0);
        assert_eq!(result, vec![f.t_string()]);
    });
}

#[test]
fn callable_string_with_literal_keeps_truthy_axis() {
    fixture(|f| {
        let lit = f.t_lit_string("foo");
        let result = combine_default(f, vec![f.t_callable_string(), lit]);
        assert_eq!(result, vec![f.t_truthy_string()]);
    });
}

#[test]
fn unspec_literal_absorbs_specific_literal() {
    fixture(|f| {
        let lit = f.t_lit_string("hi");
        let result = combine_default(f, vec![f.t_unspec_lit_string(false), lit]);
        assert_eq!(result, vec![f.t_unspec_lit_string(false)]);
    });
}

#[test]
fn non_empty_unspec_literal_absorbs_specific() {
    fixture(|f| {
        let lit = f.t_lit_string("hi");
        let result = combine_default(f, vec![f.t_unspec_lit_string(true), lit]);
        assert_eq!(result, vec![f.t_unspec_lit_string(true)]);
    });
}

#[test]
fn many_literals_with_general_collapse() {
    fixture(|f| {
        let mut inputs = vec![f.t_string()];
        for i in 0..50 {
            inputs.push(f.t_lit_string(&format!("s{i}")));
        }
        assert_combines_to(f, inputs, vec![f.t_string()]);
    });
}

#[test]
fn many_compatible_literals_with_non_empty_collapse() {
    fixture(|f| {
        let mut inputs = vec![f.t_non_empty_string()];
        for i in 0..30 {
            inputs.push(f.t_lit_string(&format!("s{i}")));
        }
        assert_combines_to(f, inputs, vec![f.t_non_empty_string()]);
    });
}

#[test]
fn mixed_compatible_and_incompatible_literals_with_non_empty() {
    fixture(|f| {
        let inputs = vec![
            f.t_non_empty_string(),
            f.t_lit_string("a"),
            f.t_lit_string("b"),
            f.t_lit_string(""),
            f.t_lit_string("c"),
        ];
        let result = combine_default(f, inputs);
        assert!(result.contains(&f.t_non_empty_string()));
        assert!(result.contains(&f.t_lit_string("")));
    });
}

#[test]
fn literal_only_combine_order_independent() {
    fixture(|f| {
        let cases = [
            vec![f.t_lit_string("a"), f.t_lit_string("b"), f.t_lit_string("c")],
            vec![f.t_lit_string("hello"), f.t_lit_string("world"), f.t_lit_string("foo")],
            vec![f.t_lit_string(""), f.t_lit_string("0"), f.t_lit_string("x")],
        ];
        for case in cases {
            let r1 = combine_default(f, case.clone());
            let mut reversed = case;
            reversed.reverse();
            let r2 = combine_default(f, reversed);
            assert_multiset_eq(&r1, &r2);
        }
    });
}

#[test]
fn lit_lower_with_lit_upper_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_lit_string("abc"), f.t_lit_string("ABC")];
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn many_lower_literals_with_non_empty_lower_collapse() {
    fixture(|f| {
        let inputs: Vec<_> =
            core::iter::once(f.t_lower_string()).chain((0..20).map(|i| f.t_lit_string(&format!("s{i}")))).collect();
        assert_combines_to(f, inputs, vec![f.t_lower_string()]);
    });
}

#[test]
fn many_uppercase_literals_with_uppercase_collapse() {
    fixture(|f| {
        let inputs: Vec<_> =
            core::iter::once(f.t_upper_string()).chain((0..20).map(|i| f.t_lit_string(&format!("S{i}")))).collect();
        assert_combines_to(f, inputs, vec![f.t_upper_string()]);
    });
}
