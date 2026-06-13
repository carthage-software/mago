mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayFlag;
use mago_oracle::ty::atom::payload::array::KnownElement;
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::join;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::refines;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::ty::well_known;
use mago_oracle::world::Variance;
use mago_oracle::world::World;

fn t_sealed_list<'arena>(f: &mut Fixture<'_, 'arena>, elements: &[Type<'arena>]) -> Atom<'arena> {
    let entries: Vec<KnownElement<'arena>> = elements
        .iter()
        .enumerate()
        .map(|(index, &value)| KnownElement { index: index as u32, value, optional: false })
        .collect();

    f.builder.sealed_list(&entries, true)
}

fn meet_of<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> Type<'arena>
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    meet::compute(a, b, world, LatticeOptions::default(), &mut report, &mut f.builder)
}

fn join_of<'arena>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>) -> Type<'arena> {
    let mut atoms = a.atoms.to_vec();
    atoms.extend_from_slice(b.atoms);
    let merged = join::compute(&atoms, &mut f.builder);
    f.builder.union_of(&merged)
}

#[track_caller]
fn assert_partition<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W)
where
    W: World<'arena>,
{
    let inside = meet_of(f, a, b, world);
    let outside = subtract_of(f, a, b, world);
    let rejoined = join_of(f, inside, outside);
    assert_eq!(rejoined, a, "partition law: join(meet(a,b), subtract(a,b)) must reconstruct a");
}

fn subtract_of<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> Type<'arena>
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    subtract::compute(a, b, world, LatticeOptions::default(), &mut report, &mut f.builder)
}

fn refines_of<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> bool
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    refines(a, b, world, LatticeOptions::default(), &mut report, &mut f.builder)
}

#[track_caller]
fn assert_upper_bound<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W)
where
    W: World<'arena>,
{
    let r = subtract_of(f, a, b, world);
    assert!(refines_of(f, r, a, world), "subtract({a:?}, {b:?}) = {r:?} does not refine {a:?}");
}

#[test]
fn reflexive_subtract_yields_never() {
    fixture(|f| {
        let cb = empty_world();
        assert_eq!(subtract_of(f, well_known::TYPE_INT, well_known::TYPE_INT, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn subtract_never_is_identity() {
    fixture(|f| {
        let cb = empty_world();
        assert_eq!(subtract_of(f, well_known::TYPE_INT, well_known::TYPE_NEVER, &cb), well_known::TYPE_INT);
    });
}

#[test]
fn subtract_from_never_yields_never() {
    fixture(|f| {
        let cb = empty_world();
        assert_eq!(subtract_of(f, well_known::TYPE_NEVER, well_known::TYPE_INT, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn subtract_mixed_yields_never() {
    fixture(|f| {
        let cb = empty_world();
        assert_eq!(subtract_of(f, well_known::TYPE_INT, well_known::TYPE_MIXED, &cb), well_known::TYPE_NEVER);
        assert_eq!(subtract_of(f, well_known::TYPE_STRING, well_known::TYPE_MIXED, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn subsumption_collapses_to_never() {
    fixture(|f| {
        let cb = empty_world();
        let lit = f.ui(42);
        assert_eq!(subtract_of(f, lit, well_known::TYPE_INT, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn disjoint_kinds_subtract_is_identity() {
    fixture(|f| {
        let cb = empty_world();
        assert_eq!(subtract_of(f, well_known::TYPE_INT, well_known::TYPE_STRING, &cb), well_known::TYPE_INT);
        assert_eq!(subtract_of(f, well_known::TYPE_STRING, well_known::TYPE_NULL, &cb), well_known::TYPE_STRING);
    });
}

#[test]
fn nullable_int_minus_null_is_int() {
    fixture(|f| {
        let cb = empty_world();
        let null = f.null();
        let int = f.t_int();
        let nullable_int = f.u_many(vec![null, int]);
        assert_eq!(subtract_of(f, nullable_int, well_known::TYPE_NULL, &cb), well_known::TYPE_INT);
    });
}

#[test]
fn union_minus_union_distributes() {
    fixture(|f| {
        let cb = empty_world();
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        let int_or_string_or_null = f.u_many(vec![int, string, null]);
        let null_or_int = f.u_many(vec![null, int]);
        assert_eq!(subtract_of(f, int_or_string_or_null, null_or_int, &cb), well_known::TYPE_STRING);
    });
}

#[test]
fn int_range_minus_literal_in_middle_splits() {
    fixture(|f| {
        let cb = empty_world();
        let range = f.t_int_range(0, 10);
        let r = f.u(range);
        let lit = f.ui(5);
        let result = subtract_of(f, r, lit, &cb);
        let low = f.t_int_range(0, 4);
        let high = f.t_int_range(6, 10);
        let expected = f.u_many(vec![low, high]);
        assert_eq!(result, expected);
    });
}

#[test]
fn int_range_minus_overlapping_range_keeps_outer_pieces() {
    fixture(|f| {
        let cb = empty_world();
        let outer_range = f.t_int_range(0, 20);
        let outer = f.u(outer_range);
        let inner_range = f.t_int_range(5, 15);
        let inner = f.u(inner_range);
        let result = subtract_of(f, outer, inner, &cb);
        let low = f.t_int_range(0, 4);
        let high = f.t_int_range(16, 20);
        let expected = f.u_many(vec![low, high]);
        assert_eq!(result, expected);
    });
}

#[test]
fn int_range_minus_overlapping_left_keeps_right() {
    fixture(|f| {
        let cb = empty_world();
        let range = f.t_int_range(0, 10);
        let r = f.u(range);
        let cut_range = f.t_int_range(-5, 5);
        let cut = f.u(cut_range);
        let result = subtract_of(f, r, cut, &cb);
        let expected_range = f.t_int_range(6, 10);
        let expected = f.u(expected_range);
        assert_eq!(result, expected);
    });
}

#[test]
fn int_range_minus_overlapping_right_keeps_left() {
    fixture(|f| {
        let cb = empty_world();
        let range = f.t_int_range(0, 10);
        let r = f.u(range);
        let cut_range = f.t_int_range(5, 15);
        let cut = f.u(cut_range);
        let result = subtract_of(f, r, cut, &cb);
        let expected_range = f.t_int_range(0, 4);
        let expected = f.u(expected_range);
        assert_eq!(result, expected);
    });
}

#[test]
fn int_range_minus_endpoint_collapses_to_literal_piece() {
    fixture(|f| {
        let cb = empty_world();
        let range = f.t_int_range(0, 1);
        let r = f.u(range);
        let zero = f.ui(0);
        let result = subtract_of(f, r, zero, &cb);
        let expected = f.ui(1);
        assert_eq!(result, expected);
    });
}

#[test]
fn int_range_minus_disjoint_range_is_identity() {
    fixture(|f| {
        let cb = empty_world();
        let range_a = f.t_int_range(0, 10);
        let a = f.u(range_a);
        let range_b = f.t_int_range(20, 30);
        let b = f.u(range_b);
        assert_eq!(subtract_of(f, a, b, &cb), a);
    });
}

#[test]
fn open_int_minus_bounded_range_splits_unbounded_pieces() {
    fixture(|f| {
        let cb = empty_world();
        let int = well_known::TYPE_INT;
        let middle_range = f.t_int_range(0, 10);
        let middle = f.u(middle_range);
        let result = subtract_of(f, int, middle, &cb);
        let low = f.t_int_to(-1);
        let high = f.t_int_from(11);
        let expected = f.u_many(vec![low, high]);
        assert_eq!(result, expected);
    });
}

#[test]
fn bool_minus_true_is_false() {
    fixture(|f| {
        let cb = empty_world();
        let bool_t = well_known::TYPE_BOOL;
        let true_t = well_known::TYPE_TRUE;
        assert_eq!(subtract_of(f, bool_t, true_t, &cb), well_known::TYPE_FALSE);
    });
}

#[test]
fn bool_minus_false_is_true() {
    fixture(|f| {
        let cb = empty_world();
        let bool_t = well_known::TYPE_BOOL;
        let false_t = well_known::TYPE_FALSE;
        assert_eq!(subtract_of(f, bool_t, false_t, &cb), well_known::TYPE_TRUE);
    });
}

#[test]
fn nullable_bool_minus_null_is_bool() {
    fixture(|f| {
        let cb = empty_world();
        let null = f.null();
        let bool_atom = f.t_bool();
        let nullable_bool = f.u_many(vec![null, bool_atom]);
        assert_eq!(subtract_of(f, nullable_bool, well_known::TYPE_NULL, &cb), well_known::TYPE_BOOL);
    });
}

#[test]
fn mixed_minus_null_is_value_equal_to_non_null_mixed() {
    fixture(|f| {
        let cb = empty_world();
        let result = subtract_of(f, well_known::TYPE_MIXED, well_known::TYPE_NULL, &cb);
        let nonnull = f.mixed_nonnull();
        let expected = f.u(nonnull);
        assert!(
            refines_of(f, result, expected, &cb) && refines_of(f, expected, result, &cb),
            "mixed \\ null should be value-equal to nonnull-mixed; got {result}, expected {expected}",
        );
    });
}

#[test]
fn multi_step_subtraction_chains() {
    fixture(|f| {
        let cb = empty_world();
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        let three = f.u_many(vec![int, string, null]);
        let null_string = f.u_many(vec![null, string]);
        assert_eq!(subtract_of(f, three, null_string, &cb), well_known::TYPE_INT);
    });
}

#[test]
fn upper_bound_invariant_int_split() {
    fixture(|f| {
        let cb = empty_world();
        let range = f.t_int_range(0, 10);
        let r = f.u(range);
        let mid = f.ui(5);
        assert_upper_bound(f, r, mid, &cb);
    });
}

#[test]
fn upper_bound_invariant_nullable_int_minus_null() {
    fixture(|f| {
        let cb = empty_world();
        let null = f.null();
        let int = f.t_int();
        let nullable_int = f.u_many(vec![null, int]);
        assert_upper_bound(f, nullable_int, well_known::TYPE_NULL, &cb);
    });
}

#[test]
fn unrelated_named_objects_subtract_records_exclusion_open_world() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.declare("Foo");
        w.declare("Bar");
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let bar_atom = f.t_named("Bar");
        let bar = f.u(bar_atom);
        let result = subtract_of(f, foo, bar, &w);
        assert!(refines_of(f, result, foo, &w), "subtract result must refine input Foo");
        let meet = meet::compute(result, bar, &w, LatticeOptions::default(), &mut LatticeReport::new(), &mut f.builder);
        assert_eq!(meet, well_known::TYPE_NEVER, "(Foo \\ Bar) ∩ Bar should be empty");
    });
}

#[test]
fn descendant_minus_ancestor_is_never() {
    fixture(|f| {
        let cb = MockWorld::from_edges(&[("Dog", "Animal")]);
        let dog_atom = f.t_named("Dog");
        let dog = f.u(dog_atom);
        let animal_atom = f.t_named("Animal");
        let animal = f.u(animal_atom);
        assert_eq!(subtract_of(f, dog, animal, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn ancestor_minus_descendant_records_exclusion() {
    fixture(|f| {
        let cb = MockWorld::from_edges(&[("Dog", "Animal")]);
        let dog_atom = f.t_named("Dog");
        let dog = f.u(dog_atom);
        let animal_atom = f.t_named("Animal");
        let animal = f.u(animal_atom);
        let diff = subtract_of(f, animal, dog, &cb);
        assert_ne!(diff, animal, "subtract should refine `Animal` rather than return identity");
        let meet = meet::compute(diff, dog, &cb, LatticeOptions::default(), &mut LatticeReport::new(), &mut f.builder);
        assert_eq!(meet, well_known::TYPE_NEVER, "(Animal \\ Dog) ∩ Dog should be never");
    });
}

#[test]
fn subtract_then_meet_is_disjoint_for_int_range() {
    fixture(|f| {
        let cb = empty_world();
        let range = f.t_int_range(0, 10);
        let r = f.u(range);
        let mid_range = f.t_int_range(5, 7);
        let mid = f.u(mid_range);
        let diff = subtract_of(f, r, mid, &cb);
        assert!(!overlaps(f, diff, mid, &cb));
    });
}

#[test]
fn template_with_int_or_string_minus_int_narrows_constraint_to_string() {
    fixture(|f| {
        let cb = empty_world();
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        let lhs_atom = f.t_template_of("C", "T", int_or_string);
        let lhs = f.u(lhs_atom);
        let string_ty = f.u(string);
        let expected_atom = f.t_template_of("C", "T", string_ty);
        let expected = f.u(expected_atom);
        assert_eq!(subtract_of(f, lhs, well_known::TYPE_INT, &cb), expected);
    });
}

#[test]
fn template_with_int_or_string_minus_string_narrows_constraint_to_int() {
    fixture(|f| {
        let cb = empty_world();
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        let lhs_atom = f.t_template_of("C", "T", int_or_string);
        let lhs = f.u(lhs_atom);
        let int_ty = f.u(int);
        let expected_atom = f.t_template_of("C", "T", int_ty);
        let expected = f.u(expected_atom);
        assert_eq!(subtract_of(f, lhs, well_known::TYPE_STRING, &cb), expected);
    });
}

#[test]
fn template_with_int_minus_int_is_impossible() {
    fixture(|f| {
        let cb = empty_world();
        let int = f.t_int();
        let int_ty = f.u(int);
        let lhs_atom = f.t_template_of("C", "T", int_ty);
        let lhs = f.u(lhs_atom);
        assert_eq!(subtract_of(f, lhs, well_known::TYPE_INT, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn template_with_int_minus_string_is_redundant_keeps_template() {
    fixture(|f| {
        let cb = empty_world();
        let int = f.t_int();
        let int_ty = f.u(int);
        let lhs_atom = f.t_template_of("C", "T", int_ty);
        let lhs = f.u(lhs_atom);
        assert_eq!(subtract_of(f, lhs, well_known::TYPE_STRING, &cb), lhs);
    });
}

#[test]
fn same_template_minus_same_template_with_disjoint_constraint_is_identity() {
    fixture(|f| {
        let cb = empty_world();
        let int = f.t_int();
        let int_ty = f.u(int);
        let lhs_atom = f.t_template_of("C", "T", int_ty);
        let lhs = f.u(lhs_atom);
        let string = f.t_string();
        let string_ty = f.u(string);
        let rhs_atom = f.t_template_of("C", "T", string_ty);
        let rhs = f.u(rhs_atom);
        assert_eq!(subtract_of(f, lhs, rhs, &cb), lhs);
    });
}

#[test]
fn same_template_minus_same_template_with_subset_constraint_is_impossible() {
    fixture(|f| {
        let cb = empty_world();
        let int = f.t_int();
        let int_ty = f.u(int);
        let lhs_atom = f.t_template_of("C", "T", int_ty);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_template_of("C", "T", well_known::TYPE_MIXED);
        let rhs = f.u(rhs_atom);
        assert_eq!(subtract_of(f, lhs, rhs, &cb), well_known::TYPE_NEVER);
    });
}

#[test]
fn class_string_minus_interface_string_narrows_to_intersected() {
    fixture(|f| {
        let w = empty_world();
        let a = f.u_many(vec![well_known::CLASS_STRING, well_known::INT]);
        let c = f.u(well_known::INTERFACE_STRING);

        let a_sub_c = subtract_of(f, a, c, &w);

        assert_ne!(a_sub_c, a, "class-string \\ interface-string must narrow");
        assert!(a_sub_c.atoms.contains(&well_known::INT));
    });
}

#[test]
fn array_minus_list_removes_empty_array() {
    fixture(|f| {
        let w = empty_world();
        let arr = f.builder.keyed_unsealed(well_known::TYPE_INT, well_known::TYPE_INT, false);
        let lst = f.builder.list_of(well_known::TYPE_NULL, false);
        let a = f.u(arr);
        let lst_ty = f.u(lst);

        let minus_result = subtract_of(f, a, lst_ty, &w);
        let atoms = minus_result.atoms;
        assert_eq!(atoms.len(), 1, "should have one element");

        let head_atom = if let Atom::Intersected(payload) = atoms[0] { *payload.head } else { atoms[0] };
        let Atom::Array(info) = head_atom else {
            panic!("expected an array atom, got {head_atom:?}");
        };
        assert!(info.flags.contains(ArrayFlag::NonEmpty), "result should be non-empty-array");
    });
}

#[test]
fn list_minus_array_removes_empty_when_array_allows_empty() {
    fixture(|f| {
        let w = empty_world();
        let lst = f.builder.list_of(well_known::TYPE_NULL, false);
        let arr = f.builder.keyed_unsealed(well_known::TYPE_INT, well_known::TYPE_INT, false);
        let a = f.u(lst);
        let arr_ty = f.u(arr);

        let minus_result = subtract_of(f, a, arr_ty, &w);
        let atoms = minus_result.atoms;
        assert_eq!(atoms.len(), 1, "should have one element");

        let head_atom = if let Atom::Intersected(payload) = atoms[0] { *payload.head } else { atoms[0] };
        let Atom::List(info) = head_atom else {
            panic!("expected a list atom, got {head_atom:?}");
        };
        assert!(info.flags.contains(ListFlag::NonEmpty), "result should be non-empty-list");
    });
}

#[test]
fn uninhabited_array_lhs_subtract_is_never() {
    fixture(|f| {
        let w = empty_world();
        let obj = f.t_named("A");
        let key_type = f.u(obj);
        let arr = f.builder.keyed_unsealed(key_type, well_known::TYPE_INT, true);
        let a = f.u(arr);
        let b = f.u_many(vec![well_known::INT, well_known::STRING]);

        let result = subtract_of(f, a, b, &w);
        assert_eq!(result, well_known::TYPE_NEVER, "uninhabited array should subtract to never");
    });
}

#[test]
fn sealed_list_minus_narrows_single_element() {
    fixture(|f| {
        let w = empty_world();
        let int_or_string = f.u_many(vec![well_known::INT, well_known::STRING]);
        let input_atom = t_sealed_list(f, &[int_or_string]);
        let input = f.u(input_atom);
        let removed_atom = t_sealed_list(f, &[well_known::TYPE_INT]);
        let removed = f.u(removed_atom);
        let result = subtract_of(f, input, removed, &w);
        let expected_atom = t_sealed_list(f, &[well_known::TYPE_STRING]);
        let expected = f.u(expected_atom);
        assert_eq!(
            result, expected,
            "subtracting a one-int tuple from a one-(int|string) tuple leaves a one-string tuple"
        );
    });
}

#[test]
fn sealed_list_partition_holds_single_element() {
    fixture(|f| {
        let w = empty_world();
        let int_or_string = f.u_many(vec![well_known::INT, well_known::STRING]);
        let input_atom = t_sealed_list(f, &[int_or_string]);
        let input = f.u(input_atom);
        let removed_atom = t_sealed_list(f, &[well_known::TYPE_INT]);
        let removed = f.u(removed_atom);
        assert_partition(f, input, removed, &w);
    });
}

#[test]
fn sealed_list_partition_holds_two_positions() {
    fixture(|f| {
        let w = empty_world();
        let int_or_string = f.u_many(vec![well_known::INT, well_known::STRING]);
        let input_atom = t_sealed_list(f, &[int_or_string, int_or_string]);
        let input = f.u(input_atom);
        let removed_atom = t_sealed_list(f, &[well_known::TYPE_INT, well_known::TYPE_STRING]);
        let removed = f.u(removed_atom);
        assert_partition(f, input, removed, &w);
    });
}

#[test]
fn sealed_list_partition_holds_three_positions() {
    fixture(|f| {
        let w = empty_world();
        let int_or_string = f.u_many(vec![well_known::INT, well_known::STRING]);
        let input_atom = t_sealed_list(f, &[int_or_string, int_or_string, int_or_string]);
        let input = f.u(input_atom);
        let removed_atom = t_sealed_list(f, &[well_known::TYPE_INT, well_known::TYPE_INT, well_known::TYPE_INT]);
        let removed = f.u(removed_atom);
        assert_partition(f, input, removed, &w);
    });
}

#[test]
fn sealed_list_residue_refines_input_and_avoids_removed() {
    fixture(|f| {
        let w = empty_world();
        let int_or_string = f.u_many(vec![well_known::INT, well_known::STRING]);
        let input_atom = t_sealed_list(f, &[int_or_string]);
        let input = f.u(input_atom);
        let removed_atom = t_sealed_list(f, &[well_known::TYPE_INT]);
        let removed = f.u(removed_atom);
        let result = subtract_of(f, input, removed, &w);
        assert!(refines_of(f, result, input, &w), "residue must refine the input");
        let back = meet_of(f, result, removed, &w);
        assert!(back.is_never(), "residue shares nothing with what was removed");
    });
}

#[test]
fn sealed_lists_of_different_length_keep_input() {
    fixture(|f| {
        let w = empty_world();
        let input_atom = t_sealed_list(f, &[well_known::TYPE_INT]);
        let input = f.u(input_atom);
        let removed_atom = t_sealed_list(f, &[well_known::TYPE_INT, well_known::TYPE_STRING]);
        let removed = f.u(removed_atom);
        let result = subtract_of(f, input, removed, &w);
        assert_eq!(result, input, "a 1-tuple and a 2-tuple are disjoint, so nothing is removed");
    });
}

#[test]
fn join_merges_single_position_diff_tuples() {
    fixture(|f| {
        let int_list = t_sealed_list(f, &[well_known::TYPE_INT]);
        let int_list_type = f.u(int_list);
        let string_list = t_sealed_list(f, &[well_known::TYPE_STRING]);
        let string_list_type = f.u(string_list);
        let merged = join_of(f, int_list_type, string_list_type);
        let int_or_string = f.u_many(vec![well_known::INT, well_known::STRING]);
        let expected_atom = t_sealed_list(f, &[int_or_string]);
        let expected = f.u(expected_atom);
        assert_eq!(merged, expected, "a one-int tuple joined with a one-string tuple is a one-(int|string) tuple");
    });
}

#[test]
fn join_merges_single_diff_among_shared_positions() {
    fixture(|f| {
        let a = t_sealed_list(f, &[well_known::TYPE_INT, well_known::TYPE_BOOL]);
        let a_type = f.u(a);
        let b = t_sealed_list(f, &[well_known::TYPE_STRING, well_known::TYPE_BOOL]);
        let b_type = f.u(b);
        let merged = join_of(f, a_type, b_type);
        let int_or_string = f.u_many(vec![well_known::INT, well_known::STRING]);
        let expected_atom = t_sealed_list(f, &[int_or_string, well_known::TYPE_BOOL]);
        let expected = f.u(expected_atom);
        assert_eq!(merged, expected, "two-element tuples differing only in position 0 merge at that position");
    });
}

#[test]
fn join_keeps_two_position_diff_tuples_distinct() {
    fixture(|f| {
        let a = t_sealed_list(f, &[well_known::TYPE_INT, well_known::TYPE_BOOL]);
        let a_type = f.u(a);
        let b = t_sealed_list(f, &[well_known::TYPE_STRING, well_known::TYPE_FLOAT]);
        let b_type = f.u(b);
        let merged = join_of(f, a_type, b_type);
        assert_eq!(merged.atoms.len(), 2, "differing in two positions must NOT merge (no phantom tuples)");
    });
}

#[test]
fn array_minus_list_partition_covers_a() {
    fixture(|f| {
        let w = empty_world();
        let zero_float = f.t_lit_float(0.0);
        let zero_float_type = f.u(zero_float);
        let arr = f.t_keyed_unsealed(well_known::TYPE_INT, zero_float_type, false);
        let a = f.u(arr);
        let list_int = f.t_list(well_known::TYPE_INT, false);
        let b = f.u(list_int);
        let m = meet_of(f, a, b, &w);
        let s = subtract_of(f, a, b, &w);
        let mut atoms = m.atoms.to_vec();
        atoms.extend_from_slice(s.atoms);
        let union = f.builder.union_of(&atoms);
        assert!(refines_of(f, a, union, &w), "A must refine meet(A,B) union subtract(A,B); a={a:?} m={m:?} s={s:?}");
    });
}

#[test]
fn possibly_empty_array_with_uninhabited_value_is_the_empty_container() {
    fixture(|f| {
        let mut w = MockWorld::new();
        for class in ["A", "B", "C", "D", "E"] {
            w.with_templates(class, &[("T", Variance::Invariant)]);
        }

        let d = f.t_named("D");
        let a_and_d = f.t_named_intersected("A", &[d]);
        let a_and_d_type = f.u(a_and_d);
        let zero = f.ui(0);
        let arr_a = f.t_keyed_unsealed(zero, a_and_d_type, false);
        let a = f.u(arr_a);

        let arr_b = f.t_keyed_unsealed(well_known::TYPE_STRING, well_known::TYPE_INT, false);
        let b = f.u(arr_b);
        let inside = meet_of(f, a, b, &w);
        let outside = subtract_of(f, a, b, &w);
        let mut atoms = inside.atoms.to_vec();
        atoms.extend_from_slice(outside.atoms);
        let union = f.builder.union_of(&atoms);
        assert!(
            refines_of(f, a, union, &w),
            "partition: A unrelated to D makes A&D uninhabited, so possibly-empty array<0, A&D> is the empty array \
             and must refine meet(A,B) union subtract(A,B); a={a:?} inside={inside:?} outside={outside:?}",
        );

        let empty_array = f.t_keyed_unsealed(well_known::TYPE_NEVER, well_known::TYPE_NEVER, false);
        let empty_array_type = f.u(empty_array);
        assert!(refines_of(f, a, empty_array_type, &w), "empty container must refine the canonical empty array");

        let empty_list = f.t_list(well_known::TYPE_NEVER, false);
        let empty_list_type = f.u(empty_list);
        assert!(refines_of(f, a, empty_list_type, &w), "empty container must refine the empty list (cross-family)");

        let iterable = f.t_iterable(well_known::TYPE_BOOL, well_known::TYPE_STRING);
        let iterable_type = f.u(iterable);
        assert!(refines_of(f, a, iterable_type, &w), "the empty array inhabits every iterable<K, V>");
    });
}
