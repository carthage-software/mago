mod comparator_common;

use comparator_common::*;

const ENUMS: &str = "
    <?php
    enum Suit { case Hearts; case Spades; }
    enum OtherSuit { case Hearts; case Spades; }
    enum Letter { case A; case a; }
    enum Singleton { case Only; }
    enum EmptyEnum {}
";

#[test]
fn enum_is_contained_by_all_its_cases_without_coercion() {
    let codebase = codebase_from_php(ENUMS);
    let input = u(t_enum("Suit"));
    let container = u_many(vec![t_enum_case("Suit", "Hearts"), t_enum_case("Suit", "Spades")]);

    let (is_subtype, result) = is_contained_capturing(&input, &container, &codebase);

    assert!(is_subtype);
    assert_eq!(result.type_coerced, None);
    assert_eq!(result.type_coerced_from_nested_mixed, None);
    assert_eq!(result.type_coerced_from_as_mixed, None);
    assert!(is_contained(&container, &input, &codebase));
}

#[test]
fn missing_cases_cannot_be_replaced_by_duplicates_or_unrelated_types() {
    let codebase = codebase_from_php(ENUMS);
    let input = u(t_enum("Suit"));

    for types in [
        vec![t_enum_case("Suit", "Hearts")],
        vec![t_enum_case("Suit", "Hearts"), t_enum_case("Suit", "Hearts")],
        vec![t_enum_case("Suit", "Hearts"), null()],
        vec![t_enum_case("Suit", "Hearts"), t_enum_case("OtherSuit", "Spades")],
    ] {
        assert!(!is_contained(&input, &u_many(types), &codebase));
    }
}

#[test]
fn enum_names_are_case_insensitive_but_distinct_enums_are_not_interchangeable() {
    let codebase = codebase_from_php(ENUMS);
    let input = u(t_enum("sUiT"));
    let same_enum = u_many(vec![t_enum_case("SUIT", "Hearts"), t_enum_case("suit", "Spades")]);
    let other_enum = u_many(vec![t_enum_case("OtherSuit", "Hearts"), t_enum_case("OtherSuit", "Spades")]);

    assert!(is_contained(&input, &same_enum, &codebase));
    assert!(!is_contained(&input, &other_enum, &codebase));
}

#[test]
fn enum_case_names_are_case_sensitive() {
    let codebase = codebase_from_php(ENUMS);
    let input = u(t_enum("Letter"));
    let complete = u_many(vec![t_enum_case("Letter", "A"), t_enum_case("Letter", "a")]);
    let incomplete = u_many(vec![t_enum_case("Letter", "A"), t_enum_case("Letter", "A")]);

    assert!(is_contained(&input, &complete, &codebase));
    assert!(!is_contained(&input, &incomplete, &codebase));
}

#[test]
fn singleton_enum_is_contained_by_its_only_case() {
    let codebase = codebase_from_php(ENUMS);

    assert!(is_contained(&u(t_enum("Singleton")), &u(t_enum_case("Singleton", "Only")), &codebase));
}

#[test]
fn enum_coverage_does_not_require_the_container_to_contain_only_enum_cases() {
    let codebase = codebase_from_php(ENUMS);
    let container = u_many(vec![t_enum_case("Suit", "Hearts"), t_enum_case("Suit", "Spades"), null()]);

    assert!(is_contained(&u(t_enum("Suit")), &container, &codebase));
    assert!(is_contained(&u_many(vec![t_enum("Suit"), null()]), &container, &codebase));
    assert!(!is_contained(&u_many(vec![t_enum("Suit"), t_int()]), &container, &codebase));
}

#[test]
fn enum_without_metadata_is_not_assumed_to_be_exhaustively_covered() {
    let codebase = empty_codebase();
    let container = u_many(vec![t_enum_case("Unknown", "A"), t_enum_case("Unknown", "B")]);

    assert!(!is_contained(&u(t_enum("Unknown")), &container, &codebase));
}

#[test]
fn empty_enum_does_not_make_unrelated_containment_vacuously_true() {
    let codebase = codebase_from_php(ENUMS);
    let container = u_many(vec![t_enum_case("Suit", "Hearts"), t_enum_case("Suit", "Spades")]);

    assert!(!is_contained(&u(t_enum("EmptyEnum")), &container, &codebase));
    assert!(!is_contained(&u(t_enum("EmptyEnum")), &u(t_enum_case("EmptyEnum", "Unknown")), &codebase));
}

#[test]
fn exhaustive_enum_case_unions_resolve_aliases_on_both_sides() {
    let codebase = codebase_from_php(
        "<?php
        enum Suit { case Hearts; case Spades; }
        enum OtherSuit { case Hearts; case Spades; }
        class_alias(Suit::class, SuitAlias::class);
        class_alias(OtherSuit::class, OtherSuitAlias::class);
        ",
    );

    for (input_name, hearts_enum, spades_enum) in
        [("SuitAlias", "Suit", "Suit"), ("Suit", "SuitAlias", "SuitAlias"), ("sUiTaLiAs", "SUIT", "suitalias")]
    {
        let input = u(t_enum(input_name));
        let container = u_many(vec![t_enum_case(hearts_enum, "Hearts"), t_enum_case(spades_enum, "Spades")]);
        let (is_subtype, result) = is_contained_capturing(&input, &container, &codebase);

        assert!(is_subtype);
        assert_eq!(result.type_coerced, None);
        assert_eq!(result.type_coerced_from_nested_mixed, None);
        assert_eq!(result.type_coerced_from_as_mixed, None);
    }

    let incomplete = u_many(vec![t_enum_case("SuitAlias", "Hearts"), t_enum_case("OtherSuitAlias", "Spades")]);
    assert!(!is_contained(&u(t_enum("SuitAlias")), &incomplete, &codebase));
}

#[test]
fn enum_cases_are_not_contained_by_other_cases_with_matching_enum_or_case_names() {
    let codebase = codebase_from_php(ENUMS);
    let input = u(t_enum_case("Suit", "Hearts"));
    let container = u_many(vec![t_enum_case("Suit", "Spades"), t_enum_case("OtherSuit", "Hearts")]);

    assert!(!is_contained(&input, &container, &codebase));
}
