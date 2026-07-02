mod common;

use common::*;

#[test]
fn string_reflexive() {
    fixture(|f| {
        let string = f.t_string();
        assert_atomic_subtype(f, string, string);
    });
}

#[test]
fn lit_reflexive_for_many_values() {
    fixture(|f| {
        for text in ["", "hi", "abc", "0", "Hello", "FOO", "foo bar", "123"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, literal);
        }
    });
}

#[test]
fn string_contains_every_literal() {
    fixture(|f| {
        let string = f.t_string();
        for text in ["", "a", "hi", "Hello", "0", "123", "foo bar", "X"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, string);
        }
    });
}

#[test]
fn string_does_not_contain_specific_literal() {
    fixture(|f| {
        let string = f.t_string();
        for text in ["a", "hi", "abc"] {
            let literal = f.t_lit_string(text);
            assert_atomic_not_subtype(f, string, literal);
        }
    });
}

#[test]
fn distinct_lits_are_disjoint() {
    fixture(|f| {
        for (left_text, right_text) in [("a", "b"), ("hi", "hello"), ("", "x"), ("Hello", "hello"), ("0", "1")] {
            let left = f.t_lit_string(left_text);
            let right = f.t_lit_string(right_text);
            assert_atomic_not_subtype(f, left, right);
        }
    });
}

#[test]
fn non_empty_in_string() {
    fixture(|f| {
        let non_empty = f.t_non_empty_string();
        let string = f.t_string();
        assert_atomic_subtype(f, non_empty, string);
    });
}

#[test]
fn string_not_in_non_empty() {
    fixture(|f| {
        let string = f.t_string();
        let non_empty = f.t_non_empty_string();
        assert_atomic_not_subtype(f, string, non_empty);
    });
}

#[test]
fn empty_lit_not_in_non_empty() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let non_empty = f.t_non_empty_string();
        assert_atomic_not_subtype(f, empty, non_empty);
    });
}

#[test]
fn non_empty_lit_in_non_empty() {
    fixture(|f| {
        let non_empty = f.t_non_empty_string();
        for text in ["a", "hi", "0", "Hello", "X"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, non_empty);
        }
    });
}

#[test]
fn truthy_in_string() {
    fixture(|f| {
        let truthy = f.t_truthy_string();
        let string = f.t_string();
        assert_atomic_subtype(f, truthy, string);
    });
}

#[test]
fn truthy_in_non_empty() {
    fixture(|f| {
        let truthy = f.t_truthy_string();
        let non_empty = f.t_non_empty_string();
        assert_atomic_subtype(f, truthy, non_empty);
    });
}

#[test]
fn non_empty_not_in_truthy() {
    fixture(|f| {
        let non_empty = f.t_non_empty_string();
        let truthy = f.t_truthy_string();
        assert_atomic_not_subtype(f, non_empty, truthy);
    });
}

#[test]
fn string_not_in_truthy() {
    fixture(|f| {
        let string = f.t_string();
        let truthy = f.t_truthy_string();
        assert_atomic_not_subtype(f, string, truthy);
    });
}

#[test]
fn truthy_lits_in_truthy_string() {
    fixture(|f| {
        let truthy = f.t_truthy_string();
        for text in ["1", "hi", "abc", "Hello", "true"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, truthy);
        }
    });
}

#[test]
fn falsy_lit_zero_not_in_truthy() {
    fixture(|f| {
        let zero = f.t_lit_string("0");
        let truthy = f.t_truthy_string();
        assert_atomic_not_subtype(f, zero, truthy);
    });
}

#[test]
fn falsy_lit_empty_not_in_truthy() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let truthy = f.t_truthy_string();
        assert_atomic_not_subtype(f, empty, truthy);
    });
}

#[test]
fn lower_in_string() {
    fixture(|f| {
        let lower = f.t_lower_string();
        let string = f.t_string();
        assert_atomic_subtype(f, lower, string);
    });
}

#[test]
fn upper_in_string() {
    fixture(|f| {
        let upper = f.t_upper_string();
        let string = f.t_string();
        assert_atomic_subtype(f, upper, string);
    });
}

#[test]
fn lower_lits_in_lower_string() {
    fixture(|f| {
        let lower = f.t_lower_string();
        for text in ["a", "hi", "abc", "hello world", "0", ""] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, lower);
        }
    });
}

#[test]
fn upper_lits_not_in_lower_string() {
    fixture(|f| {
        let lower = f.t_lower_string();
        for text in ["A", "HI", "ABC", "Hello", "World"] {
            let literal = f.t_lit_string(text);
            assert_atomic_not_subtype(f, literal, lower);
        }
    });
}

#[test]
fn upper_lits_in_upper_string() {
    fixture(|f| {
        let upper = f.t_upper_string();
        for text in ["A", "HI", "ABC", "HELLO WORLD", "0", ""] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, upper);
        }
    });
}

#[test]
fn lower_lits_not_in_upper_string() {
    fixture(|f| {
        let upper = f.t_upper_string();
        for text in ["a", "hi", "abc", "Hello"] {
            let literal = f.t_lit_string(text);
            assert_atomic_not_subtype(f, literal, upper);
        }
    });
}

#[test]
fn lower_not_in_upper() {
    fixture(|f| {
        let lower = f.t_lower_string();
        let upper = f.t_upper_string();
        assert_atomic_not_subtype(f, lower, upper);
    });
}

#[test]
fn upper_not_in_lower() {
    fixture(|f| {
        let upper = f.t_upper_string();
        let lower = f.t_lower_string();
        assert_atomic_not_subtype(f, upper, lower);
    });
}

#[test]
fn numeric_in_string() {
    fixture(|f| {
        let numeric_string = f.t_numeric_string();
        let string = f.t_string();
        assert_atomic_subtype(f, numeric_string, string);
    });
}

#[test]
fn string_not_in_numeric_string() {
    fixture(|f| {
        let string = f.t_string();
        let numeric_string = f.t_numeric_string();
        assert_atomic_not_subtype(f, string, numeric_string);
    });
}

#[test]
fn numeric_lits_in_numeric_string() {
    fixture(|f| {
        let numeric_string = f.t_numeric_string();
        for text in ["0", "1", "-1", "123", "1.5", "1e10", "-3.14", "0.5"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, numeric_string);
        }
    });
}

#[test]
fn non_numeric_lits_not_in_numeric_string() {
    fixture(|f| {
        let numeric_string = f.t_numeric_string();
        for text in ["abc", "hi", "", "12abc", "abc123"] {
            let literal = f.t_lit_string(text);
            assert_atomic_not_subtype(f, literal, numeric_string);
        }
    });
}

#[test]
fn numeric_in_non_empty_string() {
    fixture(|f| {
        let numeric_string = f.t_numeric_string();
        let non_empty = f.t_non_empty_string();
        assert_atomic_subtype(f, numeric_string, non_empty);
    });
}

#[test]
fn non_empty_not_in_numeric() {
    fixture(|f| {
        let non_empty = f.t_non_empty_string();
        let numeric_string = f.t_numeric_string();
        assert_atomic_not_subtype(f, non_empty, numeric_string);
    });
}

#[test]
fn class_string_in_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let string = f.t_string();
        assert_atomic_subtype(f, class_string, string);
    });
}

#[test]
fn interface_string_in_string() {
    fixture(|f| {
        let interface_string = f.t_interface_string();
        let string = f.t_string();
        assert_atomic_subtype(f, interface_string, string);
    });
}

#[test]
fn enum_string_in_string() {
    fixture(|f| {
        let enum_string = f.t_enum_string();
        let string = f.t_string();
        assert_atomic_subtype(f, enum_string, string);
    });
}

#[test]
fn lit_class_string_in_class_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        for text in ["Foo", "App\\Bar", "Vendor\\Pkg\\X"] {
            let literal = f.t_lit_class_string(text);
            assert_atomic_subtype(f, literal, class_string);
        }
    });
}

#[test]
fn class_string_not_in_lit_class_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let literal = f.t_lit_class_string("Foo");
        assert_atomic_not_subtype(f, class_string, literal);
    });
}

#[test]
fn unspec_lit_string_in_string() {
    fixture(|f| {
        let literal_string = f.t_unspec_lit_string(false);
        let string = f.t_string();
        assert_atomic_subtype(f, literal_string, string);
    });
}

#[test]
fn non_empty_unspec_lit_in_non_empty_string() {
    fixture(|f| {
        let literal_string = f.t_unspec_lit_string(true);
        let non_empty = f.t_non_empty_string();
        assert_atomic_subtype(f, literal_string, non_empty);
    });
}

#[test]
fn unspec_lit_not_in_non_empty() {
    fixture(|f| {
        let literal_string = f.t_unspec_lit_string(false);
        let non_empty = f.t_non_empty_string();
        assert_atomic_not_subtype(f, literal_string, non_empty);
    });
}

#[test]
fn lit_in_unspec_lit() {
    fixture(|f| {
        let literal_string = f.t_unspec_lit_string(false);
        for text in ["", "hi", "abc"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, literal_string);
        }
    });
}

#[test]
fn non_empty_lit_in_non_empty_unspec_lit() {
    fixture(|f| {
        let literal_string = f.t_unspec_lit_string(true);
        for text in ["a", "hi", "abc"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, literal_string);
        }
    });
}

#[test]
fn empty_lit_not_in_non_empty_unspec_lit() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let literal_string = f.t_unspec_lit_string(true);
        assert_atomic_not_subtype(f, empty, literal_string);
    });
}

#[test]
fn string_not_in_unspec_lit_string() {
    fixture(|f| {
        let string = f.t_string();
        let literal_string = f.t_unspec_lit_string(false);
        assert_atomic_not_subtype(f, string, literal_string);
    });
}

#[test]
fn many_distinct_lits_disjoint() {
    fixture(|f| {
        let texts: Vec<String> = (0..20).map(|index| format!("lit_{index}")).collect();
        for left_text in &texts {
            for right_text in &texts {
                if left_text == right_text {
                    continue;
                }

                let left = f.t_lit_string(left_text);
                let right = f.t_lit_string(right_text);
                assert_atomic_not_subtype(f, left, right);
            }
        }
    });
}

#[test]
fn lits_with_diff_case_disjoint() {
    fixture(|f| {
        for (left_text, right_text) in [("hello", "Hello"), ("world", "World"), ("foo", "FOO")] {
            let left = f.t_lit_string(left_text);
            let right = f.t_lit_string(right_text);
            assert_atomic_not_subtype(f, left, right);
            assert_atomic_not_subtype(f, right, left);
        }
    });
}
