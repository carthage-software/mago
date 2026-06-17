mod common;

use common::assert_multiset_eq;
use common::empty_world;
use common::fixture;

use mago_oracle::symbol::part::generic::Variance;
use mago_oracle::ty::Atom;
use mago_oracle::ty::well_known;
use mago_oracle::world::EnumBacking;
use mago_oracle::world::World;

#[test]
fn factories_produce_well_known_atoms() {
    fixture(|f| {
        assert_eq!(f.t_int(), well_known::INT);
        assert_eq!(f.t_string(), well_known::STRING);
        assert_eq!(f.null(), well_known::NULL);
        assert_eq!(f.mixed(), well_known::MIXED);
        assert_eq!(f.t_positive_int(), well_known::POSITIVE_INT);
        assert_eq!(f.t_callable_any(), well_known::CALLABLE);
    });
}

#[test]
fn unions_compose_through_the_fixture() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![string, int]);

        assert!(int_or_string.ptr_eq(&well_known::TYPE_INT_OR_STRING));
        assert_eq!(int_or_string.to_string(), "int|string");
    });
}

#[test]
fn string_literal_derives_refinements() {
    fixture(|f| {
        assert_eq!(f.t_lit_string("hello").to_string(), "string('hello')");

        let (Atom::String(hello), Atom::String(zero), Atom::String(empty), Atom::String(numeric)) =
            (f.t_lit_string("hello"), f.t_lit_string("0"), f.t_lit_string(""), f.t_lit_string("123"))
        else {
            panic!("string literals must be string atoms");
        };

        use mago_oracle::ty::atom::payload::scalar::string::StringCasing;
        use mago_oracle::ty::atom::payload::scalar::string::StringRefinementFlag;

        assert!(hello.flags.contains(StringRefinementFlag::NonEmpty));
        assert!(hello.flags.contains(StringRefinementFlag::Truthy));
        assert!(!hello.flags.contains(StringRefinementFlag::Numeric));
        assert_eq!(hello.casing, StringCasing::Lowercase);

        assert!(zero.flags.contains(StringRefinementFlag::NonEmpty));
        assert!(!zero.flags.contains(StringRefinementFlag::Truthy));
        assert!(zero.flags.contains(StringRefinementFlag::Numeric));

        assert!(!empty.flags.contains(StringRefinementFlag::NonEmpty));
        assert!(!empty.flags.contains(StringRefinementFlag::Truthy));

        assert!(numeric.flags.contains(StringRefinementFlag::Numeric));
        assert!(numeric.flags.contains(StringRefinementFlag::Truthy));
        assert_eq!(numeric.casing, StringCasing::Unspecified);
    });
}

#[test]
fn intersected_collapses_contradictions_to_never() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let foo_type = f.u(foo);
        let not_foo = f.builder.negated(foo_type);

        assert_eq!(f.builder.intersected(foo, &[not_foo]), well_known::NEVER);

        let bar = f.t_named("Bar");
        let bar_type = f.u(bar);
        let not_bar = f.builder.negated(bar_type);
        assert_eq!(f.builder.intersected(foo, &[bar, not_bar]), well_known::NEVER);
    });
}

#[test]
fn intersected_flattens_nested_heads_and_drops_self() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        let qux = f.t_named("Qux");

        let inner = f.builder.intersected(foo, &[bar]);
        let flattened = f.builder.intersected(inner, &[qux]);
        let direct = f.builder.intersected(foo, &[bar, qux]);
        assert_eq!(flattened, direct);

        let self_dropped = f.builder.intersected(foo, &[foo]);
        assert_eq!(self_dropped, foo);
    });
}

#[test]
fn negated_universal_collapses() {
    fixture(|f| {
        let never = f.builder.union_of(&[]);
        assert_eq!(f.builder.negated(never), well_known::MIXED);

        let mixed = f.u(well_known::MIXED);
        assert_eq!(f.builder.negated(mixed), well_known::NEVER);

        let int = f.u(well_known::INT);
        let not_int = f.builder.negated(int);
        let not_int_type = f.u(not_int);
        assert_eq!(f.builder.negated(not_int_type), well_known::INT);
    });
}

#[test]
fn structured_factories_render_like_suffete() {
    fixture(|f| {
        let int = f.u(well_known::INT);
        let string = f.u(well_known::STRING);

        let list = f.t_list(int, false);
        assert_eq!(list.to_string(), "list<int>");

        let array = f.t_keyed_unsealed(string, int, true);
        assert_eq!(array.to_string(), "non-empty-array<string, int>");

        let callable = f.t_callable(&[int], string);
        assert_eq!(callable.to_string(), "(callable(int): string)");

        let shape = f.t_object_shape(&[("name", string, false)], true);
        assert_eq!(shape.to_string(), "object{name: string}");

        let boxed = f.t_generic_named("Box", vec![int]);
        assert_eq!(boxed.to_string(), "Box<int>");

        let template = f.t_template("Container", "T");
        assert_eq!(template.to_string(), "'T.Container extends mixed");
    });
}

#[test]
fn sealed_list_uses_never_rest_type() {
    fixture(|f| {
        let int = f.u(well_known::INT);
        let entries = [mago_oracle::ty::atom::payload::array::KnownElement { index: 0, value: int, optional: false }];
        let sealed = f.builder.sealed_list(&entries, true);

        let Atom::List(payload) = sealed else {
            panic!("sealed list must be a list atom");
        };
        assert!(payload.element_type.is_never());
        assert_eq!(payload.known_count.map(|count| count.get()), Some(1));
        assert_eq!(sealed.to_string(), "list{0: int}");
    });
}

#[test]
fn assert_multiset_eq_ignores_order() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        assert_multiset_eq(&[int, string], &[string, int]);
    });
}

#[test]
fn mock_world_answers_hierarchy_queries() {
    fixture(|f| {
        f.world.add_edge("Child", "Parent").add_edge("Parent", "GrandParent");
        f.world.add_trait_use("Child", "Helper");

        let child = f.name("Child");
        let parent = f.name("Parent");
        let grand_parent = f.name("GrandParent");
        let helper = f.name("Helper");
        let stranger = f.name("Stranger");

        assert!(f.world.descends_from(child.id, child.id));
        assert!(f.world.descends_from(child.id, parent.id));
        assert!(f.world.descends_from(child.id, grand_parent.id));
        assert!(!f.world.descends_from(parent.id, child.id));
        assert!(!f.world.descends_from(child.id, stranger.id));

        assert!(f.world.uses_trait(child.id, helper.id));
        assert!(!f.world.uses_trait(parent.id, helper.id));
    });
}

#[test]
fn mock_world_answers_template_queries() {
    fixture(|f| {
        f.world.with_templates("Container", &[("K", Variance::Invariant), ("V", Variance::Covariant)]);
        let int = f.u(well_known::INT);
        let string = f.u(well_known::STRING);
        f.world.with_extended("IntMap", "Container", vec![int, string]);

        let container = f.name("Container");
        let int_map = f.name("IntMap");
        let v = f.builder.intern(b"V");

        assert_eq!(f.world.template_parameter_arity(container.id), 2);
        assert_eq!(f.world.template_parameter_index(container.id, v), Some(1));

        let Some(parameter) = f.world.template_parameter_at(container.id, 1) else {
            panic!("Container must declare a second template parameter");
        };
        assert_eq!(parameter.variance, Variance::Covariant);

        assert_eq!(f.world.inherited_template_argument(int_map.id, container.id, 0), Some(int));
        assert_eq!(f.world.inherited_template_argument(int_map.id, container.id, 1), Some(string));
    });
}

#[test]
fn mock_world_answers_member_queries() {
    fixture(|f| {
        let int = f.u(well_known::INT);
        let string = f.u(well_known::STRING);

        f.world.with_method("Foo", "run");
        f.world.with_property("Foo", "count", int);
        f.world.add_edge("Bar", "Foo");
        f.world.with_property("Bar", "label", string);
        f.world.with_backed_enum("Suit", string);
        f.world.with_sealed(&mut f.builder, "Shape", &["Circle", "Square"]);

        let foo = f.name("Foo");
        let bar = f.name("Bar");
        let run = f.builder.intern(b"run");
        let count = f.builder.intern(b"count");
        let label = f.builder.intern(b"label");
        let suit = f.name("Suit");
        let shape = f.name("Shape");
        let circle = f.name("Circle");

        assert!(f.world.class_has_method(foo.id, run));
        assert!(f.world.class_has_method(bar.id, run));
        assert_eq!(f.world.class_property_type(bar.id, count), Some(int));
        assert!(f.world.class_has_property(bar.id, label));
        assert_eq!(f.world.class_property_count(bar.id), 2);

        assert_eq!(f.world.enum_backing(suit.id), Some(EnumBacking::Backed(string)));
        assert!(f.world.is_final(suit.id));

        let Some(inheritors) = f.world.sealed_direct_inheritors(shape.id) else {
            panic!("Shape must be sealed");
        };
        assert_eq!(inheritors.len(), 2);
        assert_eq!(f.world.sealed_parent_of(circle.id).map(|name| name.as_bytes().to_vec()), Some(b"Shape".to_vec()));
    });
}

#[test]
fn null_world_knows_nothing() {
    fixture(|f| {
        let world = empty_world();
        let foo = f.name("Foo");
        let bar = f.name("Bar");

        assert!(!World::descends_from(&world, foo.id, bar.id));
        assert_eq!(World::template_parameter_arity(&world, foo.id), 0);
        assert_eq!(World::class_like_kind(&world, foo.id), None);
    });
}
