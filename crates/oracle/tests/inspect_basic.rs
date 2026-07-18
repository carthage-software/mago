mod common;

use common::*;

use mago_oracle::ty::AtomKind;
use mago_oracle::ty::inspect;
use mago_oracle::ty::well_known;

#[test]
fn any_top_level_match() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let ty = f.u_many(vec![int, string]);
        assert!(inspect::any(ty, |atom| atom.kind() == AtomKind::Int));
    });
}

#[test]
fn any_top_level_no_match() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let ty = f.u_many(vec![int, string]);
        assert!(!inspect::any(ty, |atom| atom.kind() == AtomKind::Float));
    });
}

#[test]
fn any_descends_into_list_element() {
    fixture(|f| {
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let list = f.u(list_atom);
        assert!(inspect::any(list, |atom| atom.kind() == AtomKind::Int));
    });
}

#[test]
fn any_descends_into_object_type_args() {
    fixture(|f| {
        let generic_atom = f.t_generic_named("Box", vec![well_known::TYPE_STRING]);
        let generic = f.u(generic_atom);
        assert!(inspect::any(generic, |atom| atom.kind() == AtomKind::String));
    });
}

#[test]
fn any_descends_into_iterable_key_and_value() {
    fixture(|f| {
        let iter_atom = f.t_iterable(well_known::TYPE_STRING, well_known::TYPE_INT);
        let iter = f.u(iter_atom);
        assert!(inspect::any(iter, |atom| atom.kind() == AtomKind::String));
        assert!(inspect::any(iter, |atom| atom.kind() == AtomKind::Int));
    });
}

#[test]
fn any_short_circuits_after_first_match() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let float = f.t_float();
        let many = f.u_many(vec![int, string, float]);
        let mut seen = 0;
        let matched = inspect::any(many, |atom| {
            seen += 1;
            atom.kind() == AtomKind::Int
        });
        assert!(matched);
        assert_eq!(seen, 1);
    });
}

#[test]
fn all_top_level_match() {
    fixture(|f| {
        let int = f.t_int();
        let forty_two = f.t_lit_int(42);
        let ty = f.u_many(vec![int, forty_two]);
        assert!(inspect::all(ty, |atom| atom.kind() == AtomKind::Int));
    });
}

#[test]
fn all_top_level_no_match() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let ty = f.u_many(vec![int, string]);
        assert!(!inspect::all(ty, |atom| atom.kind() == AtomKind::Int));
    });
}

#[test]
fn all_descends_into_list_element_failure() {
    fixture(|f| {
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let list = f.u(list_atom);
        assert!(!inspect::all(list, |atom| atom.kind() == AtomKind::Int), "the List atom itself is not Int");
    });
}

#[test]
fn any_descends_into_generic_parameter_constraint() {
    fixture(|f| {
        let param = f.t_template_of("Foo", "T", well_known::TYPE_STRING);
        let ty = f.u(param);
        assert!(inspect::any(ty, |atom| atom.kind() == AtomKind::String));
    });
}
