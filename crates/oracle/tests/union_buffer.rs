use mago_allocator::LocalArena;
use mago_flags::U16Flags;
use mago_oracle::ty::Atom;
use mago_oracle::ty::FlowFlag;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::Typed;
use mago_oracle::ty::UnionBuffer;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::well_known;

fn typed(ty: Type<'_>) -> Typed<'_> {
    Typed { ty, flags: U16Flags::empty(), meta: 0 }
}

#[test]
fn new_is_empty_with_empty_flags() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let buffer = UnionBuffer::new();
    assert!(buffer.is_empty());
    assert_eq!(buffer.flags(), U16Flags::empty());
    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_NEVER));
}

#[test]
fn default_matches_new() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let from_new = UnionBuffer::new().build(&mut builder);
    let from_default = UnionBuffer::default().build(&mut builder);
    assert_eq!(from_new, from_default);
}

#[test]
fn from_typed_round_trips_unchanged_via_short_circuit() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = Typed { ty: well_known::TYPE_INT_OR_STRING, flags: U16Flags::empty(), meta: 7 };
    let buffer = UnionBuffer::from_typed(origin);
    assert_eq!(buffer.atoms().len(), 2);

    let built = buffer.build(&mut builder);
    assert_eq!(built, origin);
    assert_eq!(built.meta, 7);
}

#[test]
fn from_via_into_round_trips() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = typed(well_known::TYPE_INT);
    let buffer: UnionBuffer<'_> = origin.into();
    assert_eq!(buffer.build(&mut builder), origin);
}

#[test]
fn push_then_build_canonicalises_to_union() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::new();
    buffer.push(well_known::INT).push(well_known::STRING);

    let built = buffer.build(&mut builder);
    assert_eq!(built, typed(well_known::TYPE_INT_OR_STRING));
    assert!(built.ty.ptr_eq(&well_known::TYPE_INT_OR_STRING));
}

#[test]
fn push_order_does_not_affect_build_result() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut forward = UnionBuffer::new();
    forward.push(well_known::INT).push(well_known::STRING);
    let mut backward = UnionBuffer::new();
    backward.push(well_known::STRING).push(well_known::INT);

    let first = forward.build(&mut builder);
    let second = backward.build(&mut builder);
    assert_eq!(first, second);
    assert!(first.ty.ptr_eq(&second.ty));
}

#[test]
fn extend_appends_many_atoms() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::new();
    buffer.extend([well_known::INT, well_known::STRING, well_known::FLOAT]);

    let expected = builder.union_of(&[well_known::INT, well_known::STRING, well_known::FLOAT]);
    assert_eq!(buffer.build(&mut builder), typed(expected));
}

#[test]
fn remove_drops_first_match() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::new();
    buffer.push(well_known::INT).push(well_known::STRING).push(well_known::INT);
    buffer.remove(well_known::INT);

    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_INT_OR_STRING));
}

#[test]
fn remove_absent_is_noop_and_preserves_short_circuit() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = Typed { ty: well_known::TYPE_INT, flags: U16Flags::empty(), meta: 7 };
    let mut buffer = UnionBuffer::from_typed(origin);
    buffer.remove(well_known::STRING);

    let built = buffer.build(&mut builder);
    assert_eq!(built, origin);
    assert_eq!(built.meta, 7);
}

#[test]
fn remove_all_drops_every_occurrence() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::new();
    buffer.push(well_known::INT).push(well_known::STRING).push(well_known::INT);
    buffer.remove_all(well_known::INT);

    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_STRING));
}

#[test]
fn retain_keeps_matching_predicate() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = builder.union_of(&[well_known::INT, well_known::STRING, well_known::FLOAT]);
    let mut buffer = UnionBuffer::from_typed(typed(origin));
    buffer.retain(|atom| *atom != well_known::STRING);

    let expected = builder.union_of(&[well_known::INT, well_known::FLOAT]);
    assert_eq!(buffer.build(&mut builder), typed(expected));
}

#[test]
fn replace_swaps_first_match() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = builder.union_of(&[well_known::NULL, well_known::INT]);
    let mut buffer = UnionBuffer::from_typed(typed(origin));
    buffer.replace(well_known::NULL, well_known::STRING);

    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_INT_OR_STRING));
}

#[test]
fn replace_absent_is_noop_and_preserves_short_circuit() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = Typed { ty: well_known::TYPE_INT, flags: U16Flags::empty(), meta: 7 };
    let mut buffer = UnionBuffer::from_typed(origin);
    buffer.replace(well_known::NULL, well_known::STRING);

    let built = buffer.build(&mut builder);
    assert_eq!(built, origin);
    assert_eq!(built.meta, 7);
}

#[test]
fn map_replaces_in_place() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let one = Atom::Int(IntAtom::Literal(1));
    let origin = builder.union_of(&[one]);
    let mut buffer = UnionBuffer::from_typed(typed(origin));
    buffer.map(|atom| if atom == one { well_known::INT } else { atom });

    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_INT));
}

#[test]
fn flat_map_one_to_many_explodes() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let five = Atom::Int(IntAtom::Literal(5));
    let low = builder.int_range_atom(Some(0), Some(4));
    let high = builder.int_range_atom(Some(6), Some(10));

    let origin = builder.union_of(&[five]);
    let mut buffer = UnionBuffer::from_typed(typed(origin));
    buffer.flat_map(|atom| if atom == five { vec![low, high] } else { vec![atom] });

    let expected = builder.union_of(&[low, high]);
    assert_eq!(buffer.build(&mut builder), typed(expected));
}

#[test]
fn flat_map_one_to_zero_drops_atom() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = well_known::TYPE_INT_OR_STRING;
    let mut buffer = UnionBuffer::from_typed(typed(origin));
    buffer.flat_map(|atom| if atom == well_known::STRING { Vec::new() } else { vec![atom] });

    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_INT));
}

#[test]
fn set_flags_replaces_flag_set() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::new();
    buffer.push(well_known::INT).set_flags(U16Flags::empty().with(FlowFlag::Populated));

    let built = buffer.build(&mut builder);
    assert!(built.flags.contains(FlowFlag::Populated));
}

#[test]
fn modify_flags_lets_caller_mutate_flag_set() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::new();
    buffer.push(well_known::INT).modify_flags(|flags| flags.with(FlowFlag::ByReference));

    let built = buffer.build(&mut builder);
    assert!(built.flags.contains(FlowFlag::ByReference));
}

#[test]
fn unmodified_from_typed_short_circuits_to_origin() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = typed(well_known::TYPE_INT_OR_STRING);
    let built = UnionBuffer::from_typed(origin).build(&mut builder);

    assert_eq!(built, origin);
    assert!(built.ty.ptr_eq(&origin.ty));
}

#[test]
fn mutated_then_reverted_buffer_still_rebuilds() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let origin = Typed { ty: well_known::TYPE_INT, flags: U16Flags::empty(), meta: 7 };
    let mut buffer = UnionBuffer::from_typed(origin);
    buffer.push(well_known::STRING).remove(well_known::STRING);

    let built = buffer.build(&mut builder);
    assert!(built.ty.ptr_eq(&origin.ty));
    assert_eq!(built.meta, 0);
}

#[test]
fn chain_of_mutations_yields_canonical_union() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::from_typed(typed(well_known::TYPE_INT));
    buffer.push(well_known::STRING).push(well_known::NULL).remove(well_known::NULL).set_flags(U16Flags::empty());

    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_INT_OR_STRING));
}

#[test]
fn contains_reports_buffer_membership() {
    let buffer = UnionBuffer::from_typed(typed(well_known::TYPE_INT_OR_STRING));

    assert!(buffer.contains(well_known::INT));
    assert!(buffer.contains(well_known::STRING));
    assert!(!buffer.contains(well_known::NULL));
}

#[test]
fn len_reports_pre_canonicalisation_count() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::new();
    buffer.push(well_known::INT).push(well_known::INT).push(well_known::STRING);
    assert_eq!(buffer.len(), 3);

    let built = buffer.build(&mut builder);
    assert_eq!(built.ty.atoms.len(), 2);
}

#[test]
fn atoms_returns_buffer_view_in_mutation_order() {
    let mut buffer = UnionBuffer::new();
    buffer.push(well_known::STRING).push(well_known::INT);

    assert_eq!(buffer.atoms(), &[well_known::STRING, well_known::INT]);
}

#[test]
fn empty_buffer_after_remove_all_collapses_to_never() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::from_typed(typed(well_known::TYPE_INT));
    buffer.remove_all(well_known::INT);

    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_NEVER));
}

#[test]
fn flat_map_collapsing_all_to_empty_yields_never() {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let mut buffer = UnionBuffer::from_typed(typed(well_known::TYPE_INT_OR_STRING));
    buffer.flat_map(|_| Vec::new());

    assert_eq!(buffer.build(&mut builder), typed(well_known::TYPE_NEVER));
}
