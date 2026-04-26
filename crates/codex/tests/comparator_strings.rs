mod comparator_common;

use comparator_common::*;

#[test]
fn string_reflexive() {
    assert_atomic_subtype(&t_string(), &t_string());
}

#[test]
fn lit_reflexive_for_many_values() {
    for s in ["", "hi", "abc", "0", "Hello", "FOO", "foo bar", "123"] {
        assert_atomic_subtype(&t_lit_string(s), &t_lit_string(s));
    }
}

#[test]
fn string_contains_every_literal() {
    for s in ["", "a", "hi", "Hello", "0", "123", "foo bar", "X"] {
        assert_atomic_subtype(&t_lit_string(s), &t_string());
    }
}

#[test]
fn string_does_not_contain_specific_literal() {
    for s in ["a", "hi", "abc"] {
        assert_atomic_not_subtype(&t_string(), &t_lit_string(s));
    }
}

#[test]
fn distinct_lits_are_disjoint() {
    for (a, b) in [("a", "b"), ("hi", "hello"), ("", "x"), ("Hello", "hello"), ("0", "1")] {
        assert_atomic_not_subtype(&t_lit_string(a), &t_lit_string(b));
    }
}

#[test]
fn non_empty_in_string() {
    assert_atomic_subtype(&t_non_empty_string(), &t_string());
}

#[test]
fn string_not_in_non_empty() {
    assert_atomic_not_subtype(&t_string(), &t_non_empty_string());
}

#[test]
fn empty_lit_not_in_non_empty() {
    assert_atomic_not_subtype(&t_lit_string(""), &t_non_empty_string());
}

#[test]
fn non_empty_lit_in_non_empty() {
    for s in ["a", "hi", "0", "Hello", "X"] {
        assert_atomic_subtype(&t_lit_string(s), &t_non_empty_string());
    }
}

#[test]
fn truthy_in_string() {
    assert_atomic_subtype(&t_truthy_string(), &t_string());
}

#[test]
fn truthy_in_non_empty() {
    assert_atomic_subtype(&t_truthy_string(), &t_non_empty_string());
}

#[test]
fn non_empty_not_in_truthy() {
    assert_atomic_not_subtype(&t_non_empty_string(), &t_truthy_string());
}

#[test]
fn string_not_in_truthy() {
    assert_atomic_not_subtype(&t_string(), &t_truthy_string());
}

#[test]
fn truthy_lits_in_truthy_string() {
    for s in ["1", "hi", "abc", "Hello", "true"] {
        assert_atomic_subtype(&t_lit_string(s), &t_truthy_string());
    }
}

#[test]
fn falsy_lit_zero_not_in_truthy() {
    assert_atomic_not_subtype(&t_lit_string("0"), &t_truthy_string());
}

#[test]
fn falsy_lit_empty_not_in_truthy() {
    assert_atomic_not_subtype(&t_lit_string(""), &t_truthy_string());
}

#[test]
fn lower_in_string() {
    assert_atomic_subtype(&t_lower_string(), &t_string());
}

#[test]
fn upper_in_string() {
    assert_atomic_subtype(&t_upper_string(), &t_string());
}

#[test]
fn lower_lits_in_lower_string() {
    for s in ["a", "hi", "abc", "hello world", "0", ""] {
        assert_atomic_subtype(&t_lit_string(s), &t_lower_string());
    }
}

#[test]
fn upper_lits_not_in_lower_string() {
    for s in ["A", "HI", "ABC", "Hello", "World"] {
        assert_atomic_not_subtype(&t_lit_string(s), &t_lower_string());
    }
}

#[test]
fn upper_lits_in_upper_string() {
    for s in ["A", "HI", "ABC", "HELLO WORLD", "0", ""] {
        assert_atomic_subtype(&t_lit_string(s), &t_upper_string());
    }
}

#[test]
fn lower_lits_not_in_upper_string() {
    for s in ["a", "hi", "abc", "Hello"] {
        assert_atomic_not_subtype(&t_lit_string(s), &t_upper_string());
    }
}

#[test]
fn lower_not_in_upper() {
    assert_atomic_not_subtype(&t_lower_string(), &t_upper_string());
}

#[test]
fn upper_not_in_lower() {
    assert_atomic_not_subtype(&t_upper_string(), &t_lower_string());
}

#[test]
fn numeric_in_string() {
    assert_atomic_subtype(&t_numeric_string(), &t_string());
}

#[test]
fn string_not_in_numeric_string() {
    assert_atomic_not_subtype(&t_string(), &t_numeric_string());
}

#[test]
fn numeric_lits_in_numeric_string() {
    for s in ["0", "1", "-1", "123", "1.5", "1e10", "-3.14", "0.5"] {
        assert_atomic_subtype(&t_lit_string(s), &t_numeric_string());
    }
}

#[test]
fn non_numeric_lits_not_in_numeric_string() {
    for s in ["abc", "hi", "", "12abc", "abc123"] {
        assert_atomic_not_subtype(&t_lit_string(s), &t_numeric_string());
    }
}

#[test]
fn numeric_in_non_empty_string() {
    assert_atomic_subtype(&t_numeric_string(), &t_non_empty_string());
}

#[test]
fn non_empty_not_in_numeric() {
    assert_atomic_not_subtype(&t_non_empty_string(), &t_numeric_string());
}

#[test]
fn class_string_in_string() {
    assert_atomic_subtype(&t_class_string(), &t_string());
}

#[test]
fn interface_string_in_string() {
    assert_atomic_subtype(&t_interface_string(), &t_string());
}

#[test]
fn enum_string_in_string() {
    assert_atomic_subtype(&t_enum_string(), &t_string());
}

#[test]
fn lit_class_string_in_class_string() {
    for s in ["Foo", "App\\Bar", "Vendor\\Pkg\\X"] {
        assert_atomic_subtype(&t_lit_class_string(s), &t_class_string());
    }
}

#[test]
fn class_string_not_in_lit_class_string() {
    assert_atomic_not_subtype(&t_class_string(), &t_lit_class_string("Foo"));
}

#[test]
fn unspec_lit_string_in_string() {
    assert_atomic_subtype(&t_unspec_lit_string(false), &t_string());
}

#[test]
fn non_empty_unspec_lit_in_non_empty_string() {
    assert_atomic_subtype(&t_unspec_lit_string(true), &t_non_empty_string());
}

#[test]
fn unspec_lit_not_in_non_empty() {
    assert_atomic_not_subtype(&t_unspec_lit_string(false), &t_non_empty_string());
}

#[test]
fn lit_in_unspec_lit() {
    for s in ["", "hi", "abc"] {
        assert_atomic_subtype(&t_lit_string(s), &t_unspec_lit_string(false));
    }
}

#[test]
fn non_empty_lit_in_non_empty_unspec_lit() {
    for s in ["a", "hi", "abc"] {
        assert_atomic_subtype(&t_lit_string(s), &t_unspec_lit_string(true));
    }
}

#[test]
fn empty_lit_not_in_non_empty_unspec_lit() {
    assert_atomic_not_subtype(&t_lit_string(""), &t_unspec_lit_string(true));
}

#[test]
fn string_not_in_unspec_lit_string() {
    assert_atomic_not_subtype(&t_string(), &t_unspec_lit_string(false));
}

#[test]
fn many_distinct_lits_disjoint() {
    let lits: Vec<_> = (0..20).map(|i| format!("lit_{i}")).collect();
    for a in &lits {
        for b in &lits {
            if a == b {
                continue;
            }
            assert_atomic_not_subtype(&t_lit_string(a), &t_lit_string(b));
        }
    }
}

#[test]
fn lits_with_diff_case_disjoint() {
    for (a, b) in [("hello", "Hello"), ("world", "World"), ("foo", "FOO")] {
        assert_atomic_not_subtype(&t_lit_string(a), &t_lit_string(b));
        assert_atomic_not_subtype(&t_lit_string(b), &t_lit_string(a));
    }
}
