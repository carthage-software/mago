mod common;

use common::*;

#[test]
fn object_any_idempotent() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.t_object_any(), n);
        }
    });
}

#[test]
fn named_object_idempotent() {
    fixture(|f| {
        for name in ["Foo", "Bar\\Baz", "Vendor\\Pkg\\Class"] {
            let named = f.t_named(name);
            for n in 1..=8 {
                assert_self_idempotent(f, named, n);
            }
        }
    });
}

#[test]
fn enum_idempotent() {
    fixture(|f| {
        for name in ["E", "MyEnum", "App\\Status"] {
            let enum_atom = f.t_enum(name);
            for n in 1..=8 {
                assert_self_idempotent(f, enum_atom, n);
            }
        }
    });
}

#[test]
fn enum_case_idempotent() {
    fixture(|f| {
        for (name, case) in [("E", "A"), ("Status", "Active"), ("Color", "Red")] {
            let case_atom = f.t_enum_case(name, case);
            for n in 1..=8 {
                assert_self_idempotent(f, case_atom, n);
            }
        }
    });
}

#[test]
fn object_any_absorbs_named() {
    fixture(|f| {
        for name in ["Foo", "Bar", "X"] {
            let named = f.t_named(name);
            assert_combines_to(f, vec![f.t_object_any(), named], vec![f.t_object_any()]);
            assert_combines_to(f, vec![named, f.t_object_any()], vec![f.t_object_any()]);
        }
    });
}

#[test]
fn object_any_absorbs_many_nameds() {
    fixture(|f| {
        let names = ["A", "B", "C", "D", "E"];
        let mut inputs = vec![f.t_object_any()];
        for n in names {
            inputs.push(f.t_named(n));
        }
        assert_combines_to(f, inputs, vec![f.t_object_any()]);
    });
}

#[test]
fn many_object_any_collapse() {
    fixture(|f| {
        for n in 1..=10 {
            assert_combines_to(f, vec![f.t_object_any(); n], vec![f.t_object_any()]);
        }
    });
}

#[test]
fn same_named_collapses() {
    fixture(|f| {
        for name in ["Foo", "App\\Bar"] {
            let named = f.t_named(name);
            for n in 1..=8 {
                assert_combines_to(f, vec![named; n], vec![named]);
            }
        }
    });
}

#[test]
fn distinct_named_kept_apart() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let r = combine_default(f, vec![foo, bar]);
        assert_eq!(r.len(), 2);
        assert!(r.contains(&foo));
        assert!(r.contains(&bar));
    });
}

#[test]
fn many_distinct_named_kept_apart() {
    fixture(|f| {
        let names = ["A", "B", "C", "D", "E", "F", "G", "H"];
        let inputs: Vec<_> = names.iter().map(|n| f.t_named(n)).collect();
        let r = combine_default(f, inputs);
        assert_eq!(r.len(), names.len());
    });
}

#[test]
fn same_generic_same_params_collapses() {
    fixture(|f| {
        for name in ["Foo", "Box", "App\\Collection"] {
            let int_argument = f.u(f.t_int());
            let first = f.t_generic_named(name, vec![int_argument]);
            let second = f.t_generic_named(name, vec![int_argument]);
            assert_eq!(first, second);
            for n in 1..=6 {
                assert_combines_to(f, vec![first; n], vec![first]);
            }
        }
    });
}

#[test]
fn same_generic_different_params_kept_separate() {
    fixture(|f| {
        let int_argument = f.u(f.t_int());
        let string_argument = f.u(f.t_string());
        let foo_int = f.t_generic_named("Foo", vec![int_argument]);
        let foo_string = f.t_generic_named("Foo", vec![string_argument]);
        let result = combine_default(f, vec![foo_int, foo_string]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&foo_int));
        assert!(result.contains(&foo_string));
    });
}

#[test]
fn different_generic_kept_apart() {
    fixture(|f| {
        let int_argument = f.u(f.t_int());
        let foo = f.t_generic_named("Foo", vec![int_argument]);
        let bar = f.t_generic_named("Bar", vec![int_argument]);
        let result = combine_default(f, vec![foo, bar]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&foo));
        assert!(result.contains(&bar));
    });
}

#[test]
fn generic_with_invariant_literal_param_kept_separate() {
    fixture(|f| {
        let int_argument = f.u(f.t_int());
        let literal_argument = f.ui(5);
        let foo_int = f.t_generic_named("Foo", vec![int_argument]);
        let foo_literal = f.t_generic_named("Foo", vec![literal_argument]);
        let result = combine_default(f, vec![foo_int, foo_literal]);
        assert_eq!(result.len(), 2);
        assert!(result.contains(&foo_int));
        assert!(result.contains(&foo_literal));
    });
}

#[test]
fn generic_with_many_distinct_params_kept_separate() {
    fixture(|f| {
        let containers: Vec<_> = (0..5)
            .map(|i| {
                let argument = f.ui(i);
                f.t_generic_named("Box", vec![argument])
            })
            .collect();
        let result = combine_default(f, containers);
        assert_eq!(result.len(), 5);
    });
}

#[test]
fn distinct_enums_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_enum("E"), f.t_enum("F")];
        let r = combine_default(f, inputs);
        assert_eq!(r.len(), 2);
    });
}

#[test]
fn same_enum_collapses() {
    fixture(|f| {
        let enum_atom = f.t_enum("E");
        assert_combines_to(f, vec![enum_atom; 5], vec![enum_atom]);
    });
}

#[test]
fn distinct_enum_cases_same_enum_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_enum_case("E", "A"), f.t_enum_case("E", "B")];
        let r = combine_default(f, inputs);
        assert_eq!(r.len(), 2);
    });
}

#[test]
fn same_enum_case_collapses() {
    fixture(|f| {
        let case_atom = f.t_enum_case("E", "A");
        assert_combines_to(f, vec![case_atom; 5], vec![case_atom]);
    });
}

#[test]
fn enum_case_absorbed_by_enum() {
    fixture(|f| {
        let enum_atom = f.t_enum("E");
        let case_atom = f.t_enum_case("E", "A");
        let r = combine_default(f, vec![enum_atom, case_atom]);
        assert_eq!(r, vec![enum_atom]);
    });
}

#[test]
fn enum_or_named_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_enum("E"), f.t_named("Foo")];
        let r = combine_default(f, inputs);
        assert_eq!(r.len(), 2);
    });
}

#[test]
fn object_or_int_kept_separate() {
    fixture(|f| {
        let r = combine_default(f, vec![f.t_object_any(), f.t_int()]);
        assert_eq!(r.len(), 2);
        assert!(r.contains(&f.t_object_any()));
        assert!(r.contains(&f.t_int()));
    });
}

#[test]
fn named_or_string_kept_separate() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let r = combine_default(f, vec![foo, f.t_string()]);
        assert_eq!(r.len(), 2);
        assert!(r.contains(&foo));
        assert!(r.contains(&f.t_string()));
    });
}

#[test]
fn many_objects_with_int_kept_separate() {
    fixture(|f| {
        let inputs = vec![f.t_named("A"), f.t_named("B"), f.t_named("C"), f.t_int()];
        let r = combine_default(f, inputs);
        assert_eq!(r.len(), 4);
    });
}

#[test]
fn object_dominated_by_mixed() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_object_any(), f.mixed()], vec![f.mixed()]);
        assert_combines_to(f, vec![f.mixed(), f.t_object_any()], vec![f.mixed()]);
        let foo = f.t_named("Foo");
        assert_combines_to(f, vec![foo, f.mixed()], vec![f.mixed()]);
    });
}
