mod common;

use common::*;

#[test]
fn resource_reflexive() {
    fixture(|f| {
        let resource = f.t_resource();
        assert_atomic_subtype(f, resource, resource);
    });
}

#[test]
fn open_reflexive() {
    fixture(|f| {
        let open = f.t_open_resource();
        assert_atomic_subtype(f, open, open);
    });
}

#[test]
fn closed_reflexive() {
    fixture(|f| {
        let closed = f.t_closed_resource();
        assert_atomic_subtype(f, closed, closed);
    });
}

#[test]
fn open_in_resource() {
    fixture(|f| {
        let open = f.t_open_resource();
        let resource = f.t_resource();
        assert_atomic_subtype(f, open, resource);
    });
}

#[test]
fn closed_in_resource() {
    fixture(|f| {
        let closed = f.t_closed_resource();
        let resource = f.t_resource();
        assert_atomic_subtype(f, closed, resource);
    });
}

#[test]
fn resource_not_in_open() {
    fixture(|f| {
        let resource = f.t_resource();
        let open = f.t_open_resource();
        assert_atomic_not_subtype(f, resource, open);
    });
}

#[test]
fn resource_not_in_closed() {
    fixture(|f| {
        let resource = f.t_resource();
        let closed = f.t_closed_resource();
        assert_atomic_not_subtype(f, resource, closed);
    });
}

#[test]
fn open_not_in_closed() {
    fixture(|f| {
        let open = f.t_open_resource();
        let closed = f.t_closed_resource();
        assert_atomic_not_subtype(f, open, closed);
    });
}

#[test]
fn closed_not_in_open() {
    fixture(|f| {
        let closed = f.t_closed_resource();
        let open = f.t_open_resource();
        assert_atomic_not_subtype(f, closed, open);
    });
}

#[test]
fn resource_not_in_int() {
    fixture(|f| {
        let resource = f.t_resource();
        let int = f.t_int();
        assert_atomic_not_subtype(f, resource, int);
    });
}

#[test]
fn resource_not_in_string() {
    fixture(|f| {
        let resource = f.t_resource();
        let string = f.t_string();
        assert_atomic_not_subtype(f, resource, string);
    });
}

#[test]
fn resource_not_in_object() {
    fixture(|f| {
        let resource = f.t_resource();
        let object = f.t_object_any();
        assert_atomic_not_subtype(f, resource, object);
    });
}

#[test]
fn resource_not_in_array() {
    fixture(|f| {
        let resource = f.t_resource();
        let array = f.t_empty_array();
        assert_atomic_not_subtype(f, resource, array);
    });
}

#[test]
fn resource_in_mixed() {
    fixture(|f| {
        let resource = f.t_resource();
        let open = f.t_open_resource();
        let closed = f.t_closed_resource();
        let mixed = f.mixed();
        assert_atomic_subtype(f, resource, mixed);
        assert_atomic_subtype(f, open, mixed);
        assert_atomic_subtype(f, closed, mixed);
    });
}

#[test]
fn never_in_resource() {
    fixture(|f| {
        let never = f.never();
        let resource = f.t_resource();
        let open = f.t_open_resource();
        let closed = f.t_closed_resource();
        assert_atomic_subtype(f, never, resource);
        assert_atomic_subtype(f, never, open);
        assert_atomic_subtype(f, never, closed);
    });
}
