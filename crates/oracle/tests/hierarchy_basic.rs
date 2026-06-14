mod common;

use common::*;

use mago_oracle::name::Name;
use mago_oracle::ty::Type;
use mago_oracle::ty::hierarchy::Hierarchy;
use mago_oracle::ty::hierarchy::HierarchyBuilder;
use mago_oracle::world::Variance;

fn build_with_template_world<'arena>(
    f: &mut Fixture<'_, 'arena>,
    edges: &[(&'static str, &'static str, Vec<Type<'arena>>)],
    templates: &[(&'static str, &'static str)],
) -> Hierarchy<'arena> {
    let mut world = MockWorld::new();
    for (class, template) in templates {
        world.with_templates(class, &[(*template, Variance::Invariant)]);
    }
    let mut hierarchy_builder = HierarchyBuilder::new();
    for (child, parent, arguments) in edges {
        let child = f.name(child);
        let parent = f.name(parent);
        hierarchy_builder.add_edge(child, parent, arguments.clone());
    }
    hierarchy_builder.build(&world, &mut f.builder)
}

#[test]
fn direct_edge_returns_registered_args() {
    fixture(|f| {
        let string = f.t_string();
        let string_type = f.u(string);
        let hierarchy = build_with_template_world(f, &[("B", "A", vec![string_type])], &[("A", "T")]);
        let child = f.name("B");
        let ancestor = f.name("A");
        assert_eq!(hierarchy.arg(child, ancestor, 0), Some(string_type));
    });
}

#[test]
fn missing_pair_returns_none() {
    fixture(|f| {
        let string = f.t_string();
        let string_type = f.u(string);
        let hierarchy = build_with_template_world(f, &[("B", "A", vec![string_type])], &[("A", "T")]);
        let child = f.name("B");
        let other = f.name("Other");
        let foo = f.name("Foo");
        let ancestor = f.name("A");
        assert_eq!(hierarchy.arg(child, other, 0), None);
        assert_eq!(hierarchy.arg(foo, ancestor, 0), None);
    });
}

#[test]
fn transitive_two_step_concrete_args_compose() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);

        let string = f.t_string();
        let string_type = f.u(string);
        let child_b = f.name("B");
        let child_c = f.name("C");
        let ancestor = f.name("A");

        let mut hierarchy_builder = HierarchyBuilder::new();
        hierarchy_builder.add_edge(child_b, ancestor, vec![string_type]);
        hierarchy_builder.add_edge(child_c, child_b, vec![]);
        let hierarchy = hierarchy_builder.build(&world, &mut f.builder);

        assert_eq!(hierarchy.arg(child_c, ancestor, 0), Some(string_type));
    });
}

#[test]
fn transitive_two_step_template_threading() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);
        world.with_templates("B", &[("U", Variance::Invariant)]);
        world.with_templates("C", &[("V", Variance::Invariant)]);

        let b_u = f.t_template("B", "U");
        let b_template = f.u(b_u);
        let c_v = f.t_template("C", "V");
        let c_template = f.u(c_v);

        let child_b = f.name("B");
        let child_c = f.name("C");
        let ancestor = f.name("A");

        let mut hierarchy_builder = HierarchyBuilder::new();
        hierarchy_builder.add_edge(child_b, ancestor, vec![b_template]);
        hierarchy_builder.add_edge(child_c, child_b, vec![c_template]);
        let hierarchy = hierarchy_builder.build(&world, &mut f.builder);

        assert_eq!(hierarchy.arg(child_c, ancestor, 0), Some(c_template), "C passes its own V through to A");
    });
}

#[test]
fn transitive_three_step_chain() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);
        world.with_templates("B", &[("U", Variance::Invariant)]);
        world.with_templates("C", &[("V", Variance::Invariant)]);

        let b_u = f.t_template("B", "U");
        let b_template = f.u(b_u);
        let c_v = f.t_template("C", "V");
        let c_template = f.u(c_v);
        let int = f.t_int();
        let int_type = f.u(int);

        let child_b = f.name("B");
        let child_c = f.name("C");
        let child_d = f.name("D");
        let ancestor = f.name("A");

        let mut hierarchy_builder = HierarchyBuilder::new();
        hierarchy_builder.add_edge(child_b, ancestor, vec![b_template]);
        hierarchy_builder.add_edge(child_c, child_b, vec![c_template]);
        hierarchy_builder.add_edge(child_d, child_c, vec![int_type]);
        let hierarchy = hierarchy_builder.build(&world, &mut f.builder);

        assert_eq!(hierarchy.arg(child_d, ancestor, 0), Some(int_type));
        assert_eq!(hierarchy.arg(child_d, child_b, 0), Some(int_type));
        assert_eq!(hierarchy.arg(child_d, child_c, 0), Some(int_type));
    });
}

#[test]
fn nested_template_in_parent_args_substitutes() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);
        world.with_templates("B", &[("U", Variance::Invariant)]);
        world.with_templates("C", &[("V", Variance::Invariant)]);

        let b_u = f.t_template("B", "U");
        let b_u_type = f.u(b_u);
        let list_of_b_u_atom = f.t_list(b_u_type, false);
        let list_of_b_u = f.u(list_of_b_u_atom);
        let c_v = f.t_template("C", "V");
        let c_v_type = f.u(c_v);
        let list_of_c_v_atom = f.t_list(c_v_type, false);
        let list_of_c_v = f.u(list_of_c_v_atom);

        let child_b = f.name("B");
        let child_c = f.name("C");
        let ancestor = f.name("A");

        let mut hierarchy_builder = HierarchyBuilder::new();
        hierarchy_builder.add_edge(child_b, ancestor, vec![list_of_b_u]);
        hierarchy_builder.add_edge(child_c, child_b, vec![c_v_type]);
        let hierarchy = hierarchy_builder.build(&world, &mut f.builder);

        assert_eq!(hierarchy.arg(child_c, ancestor, 0), Some(list_of_c_v), "C composes to passing list<V> to A");
    });
}

#[test]
fn args_returns_full_slice() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant), ("U", Variance::Invariant)]);

        let string = f.t_string();
        let string_type = f.u(string);
        let int = f.t_int();
        let int_type = f.u(int);

        let child = f.name("B");
        let ancestor = f.name("A");

        let mut hierarchy_builder = HierarchyBuilder::new();
        hierarchy_builder.add_edge(child, ancestor, vec![string_type, int_type]);
        let hierarchy = hierarchy_builder.build(&world, &mut f.builder);

        let Some(arguments) = hierarchy.args(child, ancestor) else { panic!("B must record arguments for A") };
        assert_eq!(arguments, &[string_type, int_type]);
    });
}

#[test]
fn iter_yields_every_pair() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);

        let int = f.t_int();
        let int_type = f.u(int);

        let child_b = f.name("B");
        let child_c = f.name("C");
        let ancestor = f.name("A");

        let mut hierarchy_builder = HierarchyBuilder::new();
        hierarchy_builder.add_edge(child_b, ancestor, vec![int_type]);
        hierarchy_builder.add_edge(child_c, child_b, vec![]);
        let hierarchy = hierarchy_builder.build(&world, &mut f.builder);

        let pairs: Vec<(Name<'_>, Name<'_>)> = hierarchy.iter().map(|(pair, _)| pair).collect();
        assert!(pairs.contains(&(child_b, ancestor)));
        assert!(pairs.contains(&(child_c, child_b)));
        assert!(pairs.contains(&(child_c, ancestor)));
    });
}

#[test]
fn last_added_edge_wins_on_duplicate() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);

        let int = f.t_int();
        let int_type = f.u(int);
        let string = f.t_string();
        let string_type = f.u(string);

        let child = f.name("B");
        let ancestor = f.name("A");

        let mut hierarchy_builder = HierarchyBuilder::new();
        hierarchy_builder.add_edge(child, ancestor, vec![int_type]);
        hierarchy_builder.add_edge(child, ancestor, vec![string_type]);
        let hierarchy = hierarchy_builder.build(&world, &mut f.builder);

        assert_eq!(hierarchy.arg(child, ancestor, 0), Some(string_type));
    });
}
