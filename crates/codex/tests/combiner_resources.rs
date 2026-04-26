mod combiner_common;

use combiner_common::*;

#[test]
fn idempotent_general() {
    for n in 1..=10 {
        assert_self_idempotent(t_resource(), n);
    }
}

#[test]
fn idempotent_open() {
    for n in 1..=10 {
        assert_self_idempotent(t_open_resource(), n);
    }
}

#[test]
fn idempotent_closed() {
    for n in 1..=10 {
        assert_self_idempotent(t_closed_resource(), n);
    }
}

#[test]
fn open_or_closed_is_resource() {
    for inputs in [vec![t_open_resource(), t_closed_resource()], vec![t_closed_resource(), t_open_resource()]] {
        assert_combines_to(inputs, vec![t_resource()]);
    }
}

#[test]
fn open_or_closed_or_general_is_resource() {
    for inputs in [
        vec![t_open_resource(), t_closed_resource(), t_resource()],
        vec![t_resource(), t_open_resource(), t_closed_resource()],
        vec![t_resource(), t_closed_resource(), t_open_resource()],
    ] {
        assert_combines_to(inputs, vec![t_resource()]);
    }
}

#[test]
fn general_absorbs_open() {
    for inputs in [vec![t_resource(), t_open_resource()], vec![t_open_resource(), t_resource()]] {
        assert_combines_to(inputs, vec![t_resource()]);
    }
}

#[test]
fn general_absorbs_closed() {
    for inputs in [vec![t_resource(), t_closed_resource()], vec![t_closed_resource(), t_resource()]] {
        assert_combines_to(inputs, vec![t_resource()]);
    }
}

#[test]
fn open_or_int_kept_separate() {
    let result = combine_default(vec![t_open_resource(), t_int()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["int", "open-resource"]);
}

#[test]
fn closed_or_string_kept_separate() {
    let result = combine_default(vec![t_closed_resource(), t_string()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["closed-resource", "string"]);
}

#[test]
fn resource_or_null_kept_separate() {
    let result = combine_default(vec![t_resource(), null()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["null", "resource"]);
}

#[test]
fn many_open_resources_collapse() {
    assert_combines_to(vec![t_open_resource(); 10], vec![t_open_resource()]);
}

#[test]
fn many_closed_resources_collapse() {
    assert_combines_to(vec![t_closed_resource(); 10], vec![t_closed_resource()]);
}

#[test]
fn alternating_open_closed_collapses_to_resource() {
    let inputs: Vec<_> =
        (0..10).map(|i| if i % 2 == 0 { t_open_resource() } else { t_closed_resource() }).collect();
    assert_combines_to(inputs, vec![t_resource()]);
}
