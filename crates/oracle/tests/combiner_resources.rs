mod common;

use common::*;

#[test]
fn idempotent_general() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.t_resource(), n);
        }
    });
}

#[test]
fn idempotent_open() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.t_open_resource(), n);
        }
    });
}

#[test]
fn idempotent_closed() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.t_closed_resource(), n);
        }
    });
}

#[test]
fn open_or_closed_is_resource() {
    fixture(|f| {
        for inputs in
            [vec![f.t_open_resource(), f.t_closed_resource()], vec![f.t_closed_resource(), f.t_open_resource()]]
        {
            assert_combines_to(f, inputs, vec![f.t_resource()]);
        }
    });
}

#[test]
fn open_or_closed_or_general_is_resource() {
    fixture(|f| {
        for inputs in [
            vec![f.t_open_resource(), f.t_closed_resource(), f.t_resource()],
            vec![f.t_resource(), f.t_open_resource(), f.t_closed_resource()],
            vec![f.t_resource(), f.t_closed_resource(), f.t_open_resource()],
        ] {
            assert_combines_to(f, inputs, vec![f.t_resource()]);
        }
    });
}

#[test]
fn general_absorbs_open() {
    fixture(|f| {
        for inputs in [vec![f.t_resource(), f.t_open_resource()], vec![f.t_open_resource(), f.t_resource()]] {
            assert_combines_to(f, inputs, vec![f.t_resource()]);
        }
    });
}

#[test]
fn general_absorbs_closed() {
    fixture(|f| {
        for inputs in [vec![f.t_resource(), f.t_closed_resource()], vec![f.t_closed_resource(), f.t_resource()]] {
            assert_combines_to(f, inputs, vec![f.t_resource()]);
        }
    });
}

#[test]
fn open_or_int_kept_separate() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.t_open_resource(), f.t_int()]);
        sorted.sort();
        let mut expected = vec![f.t_open_resource(), f.t_int()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn closed_or_string_kept_separate() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.t_closed_resource(), f.t_string()]);
        sorted.sort();
        let mut expected = vec![f.t_closed_resource(), f.t_string()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn resource_or_null_kept_separate() {
    fixture(|f| {
        let mut sorted = combine_default(f, vec![f.t_resource(), f.null()]);
        sorted.sort();
        let mut expected = vec![f.t_resource(), f.null()];
        expected.sort();
        assert_eq!(sorted, expected);
    });
}

#[test]
fn many_open_resources_collapse() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_open_resource(); 10], vec![f.t_open_resource()]);
    });
}

#[test]
fn many_closed_resources_collapse() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_closed_resource(); 10], vec![f.t_closed_resource()]);
    });
}

#[test]
fn alternating_open_closed_collapses_to_resource() {
    fixture(|f| {
        let inputs: Vec<_> =
            (0..10).map(|i| if i % 2 == 0 { f.t_open_resource() } else { f.t_closed_resource() }).collect();
        assert_combines_to(f, inputs, vec![f.t_resource()]);
    });
}
