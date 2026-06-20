mod common;

use common::*;

use mago_allocator::LocalArena;
use mago_oracle::symbol::SymbolTable;

fn animal_hierarchy(arena: &LocalArena) -> SymbolTable<'_, LocalArena> {
    symbol_table(
        arena,
        "<?php
class Animal {}
class Mammal extends Animal {}
class Dog extends Mammal {}
class Cat extends Mammal {}
class Cocker extends Dog {}
class Poodle extends Dog {}
interface Sleepable {}
class Sloth extends Mammal implements Sleepable {}
class Standalone {}",
    )
}

fn enums_hierarchy(arena: &LocalArena) -> SymbolTable<'_, LocalArena> {
    symbol_table(arena, "<?php enum Status {} enum Color {}")
}

#[test]
fn object_any_reflexive() {
    fixture(|f| {
        let object = f.t_object_any();
        assert!(atomic_is_contained(f, object, object, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn named_in_object_any() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let object = f.t_object_any();
        let dog = f.t_named("Dog");
        let cat = f.t_named("Cat");
        let standalone = f.t_named("Standalone");
        assert!(atomic_is_contained(f, dog, object, &symbols));
        assert!(atomic_is_contained(f, cat, object, &symbols));
        assert!(atomic_is_contained(f, standalone, object, &symbols));
    });
}

#[test]
fn object_any_not_in_named() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let object = f.t_object_any();
        let dog = f.t_named("Dog");
        assert!(!atomic_is_contained(f, object, dog, &symbols));
    });
}

#[test]
fn class_reflexive() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        for n in ["Dog", "Cat", "Cocker", "Animal", "Standalone"] {
            let named = f.t_named(n);
            assert!(atomic_is_contained(f, named, named, &symbols));
        }
    });
}

#[test]
fn subclass_in_superclass() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let cocker = f.t_named("Cocker");
        let poodle = f.t_named("Poodle");
        let dog = f.t_named("Dog");
        assert!(atomic_is_contained(f, cocker, dog, &symbols));
        assert!(atomic_is_contained(f, poodle, dog, &symbols));
    });
}

#[test]
fn superclass_not_in_subclass() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let dog = f.t_named("Dog");
        let cocker = f.t_named("Cocker");
        assert!(!atomic_is_contained(f, dog, cocker, &symbols));
    });
}

#[test]
fn class_not_in_sibling() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let cocker = f.t_named("Cocker");
        let cat = f.t_named("Cat");
        let poodle = f.t_named("Poodle");
        let dog = f.t_named("Dog");
        assert!(!atomic_is_contained(f, cocker, cat, &symbols));
        assert!(!atomic_is_contained(f, cocker, poodle, &symbols));
        assert!(!atomic_is_contained(f, dog, cat, &symbols));
    });
}

#[test]
fn class_implements_interface() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let dog = f.t_named("Dog");
        let cat = f.t_named("Cat");
        let mammal = f.t_named("Mammal");
        assert!(atomic_is_contained(f, dog, mammal, &symbols));
        assert!(atomic_is_contained(f, cat, mammal, &symbols));
    });
}

#[test]
fn class_implements_inherited_interface() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let dog = f.t_named("Dog");
        let cocker = f.t_named("Cocker");
        let animal = f.t_named("Animal");
        assert!(atomic_is_contained(f, dog, animal, &symbols));
        assert!(atomic_is_contained(f, cocker, animal, &symbols));
    });
}

#[test]
fn interface_in_parent_interface() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let mammal = f.t_named("Mammal");
        let animal = f.t_named("Animal");
        assert!(atomic_is_contained(f, mammal, animal, &symbols));
    });
}

#[test]
fn class_implements_multiple_interfaces() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let sloth = f.t_named("Sloth");
        let mammal = f.t_named("Mammal");
        let sleepable = f.t_named("Sleepable");
        let animal = f.t_named("Animal");
        assert!(atomic_is_contained(f, sloth, mammal, &symbols));
        assert!(atomic_is_contained(f, sloth, sleepable, &symbols));
        assert!(atomic_is_contained(f, sloth, animal, &symbols));
    });
}

#[test]
fn class_does_not_implement_unrelated_interface() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let dog = f.t_named("Dog");
        let cat = f.t_named("Cat");
        let standalone = f.t_named("Standalone");
        let sleepable = f.t_named("Sleepable");
        let animal = f.t_named("Animal");
        assert!(!atomic_is_contained(f, dog, sleepable, &symbols));
        assert!(!atomic_is_contained(f, cat, sleepable, &symbols));
        assert!(!atomic_is_contained(f, standalone, animal, &symbols));
    });
}

#[test]
fn parent_interface_not_in_child() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let animal = f.t_named("Animal");
        let mammal = f.t_named("Mammal");
        assert!(!atomic_is_contained(f, animal, mammal, &symbols));
    });
}

#[test]
fn interface_not_in_class() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let mammal = f.t_named("Mammal");
        let animal = f.t_named("Animal");
        let dog = f.t_named("Dog");
        assert!(!atomic_is_contained(f, mammal, dog, &symbols));
        assert!(!atomic_is_contained(f, animal, dog, &symbols));
    });
}

#[test]
fn unrelated_classes_disjoint() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let dog = f.t_named("Dog");
        let standalone = f.t_named("Standalone");
        assert!(!atomic_is_contained(f, dog, standalone, &symbols));
        assert!(!atomic_is_contained(f, standalone, dog, &symbols));
    });
}

#[test]
fn class_not_in_object_when_not_a_subtype() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let cat = f.t_named("Cat");
        let cocker = f.t_named("Cocker");
        assert!(!atomic_is_contained(f, cat, cocker, &symbols));
    });
}

#[test]
fn enum_reflexive() {
    fixture(|f| {
        let symbols = enums_hierarchy(f.arena);
        let status = f.t_enum("Status");
        let color = f.t_enum("Color");
        assert!(atomic_is_contained(f, status, status, &symbols));
        assert!(atomic_is_contained(f, color, color, &symbols));
    });
}

#[test]
fn enum_case_in_enum() {
    fixture(|f| {
        let symbols = enums_hierarchy(f.arena);
        let status = f.t_enum("Status");
        let color = f.t_enum("Color");
        let active = f.t_enum_case("Status", "Active");
        let inactive = f.t_enum_case("Status", "Inactive");
        let red = f.t_enum_case("Color", "Red");
        assert!(atomic_is_contained(f, active, status, &symbols));
        assert!(atomic_is_contained(f, inactive, status, &symbols));
        assert!(atomic_is_contained(f, red, color, &symbols));
    });
}

#[test]
fn enum_not_in_enum_case() {
    fixture(|f| {
        let symbols = enums_hierarchy(f.arena);
        let status = f.t_enum("Status");
        let active = f.t_enum_case("Status", "Active");
        assert!(!atomic_is_contained(f, status, active, &symbols));
    });
}

#[test]
fn enum_case_reflexive() {
    fixture(|f| {
        let symbols = enums_hierarchy(f.arena);
        let active = f.t_enum_case("Status", "Active");
        assert!(atomic_is_contained(f, active, active, &symbols));
    });
}

#[test]
fn enum_cases_disjoint() {
    fixture(|f| {
        let symbols = enums_hierarchy(f.arena);
        let active = f.t_enum_case("Status", "Active");
        let inactive = f.t_enum_case("Status", "Inactive");
        assert!(!atomic_is_contained(f, active, inactive, &symbols));
    });
}

#[test]
fn distinct_enums_disjoint() {
    fixture(|f| {
        let symbols = enums_hierarchy(f.arena);
        let status = f.t_enum("Status");
        let color = f.t_enum("Color");
        let active = f.t_enum_case("Status", "Active");
        assert!(!atomic_is_contained(f, status, color, &symbols));
        assert!(!atomic_is_contained(f, active, color, &symbols));
    });
}

#[test]
fn class_not_in_int() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let dog = f.t_named("Dog");
        let int = f.t_int();
        assert!(!atomic_is_contained(f, dog, int, &symbols));
    });
}

#[test]
fn class_not_in_string() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let dog = f.t_named("Dog");
        let string = f.t_string();
        assert!(!atomic_is_contained(f, dog, string, &symbols));
    });
}

#[test]
fn class_in_mixed() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let dog = f.t_named("Dog");
        let object = f.t_object_any();
        let mixed = f.mixed();
        assert!(atomic_is_contained(f, dog, mixed, &symbols));
        assert!(atomic_is_contained(f, object, mixed, &symbols));
    });
}

#[test]
fn many_class_hierarchy_relations() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let pairs_subtype = [
            ("Cocker", "Dog"),
            ("Cocker", "Animal"),
            ("Cocker", "Mammal"),
            ("Poodle", "Dog"),
            ("Dog", "Mammal"),
            ("Cat", "Mammal"),
            ("Mammal", "Animal"),
            ("Sloth", "Mammal"),
            ("Sloth", "Sleepable"),
            ("Sloth", "Animal"),
        ];
        for (sub, sup) in pairs_subtype {
            let input = f.t_named(sub);
            let container = f.t_named(sup);
            assert!(atomic_is_contained(f, input, container, &symbols), "{sub} should be a subtype of {sup}");
        }
    });
}

#[test]
fn many_class_hierarchy_non_relations() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let pairs_not_subtype = [
            ("Dog", "Cat"),
            ("Cocker", "Cat"),
            ("Cocker", "Poodle"),
            ("Animal", "Mammal"),
            ("Animal", "Dog"),
            ("Mammal", "Dog"),
            ("Dog", "Sleepable"),
            ("Cat", "Sleepable"),
            ("Standalone", "Animal"),
            ("Dog", "Standalone"),
            ("Standalone", "Dog"),
        ];
        for (sub, sup) in pairs_not_subtype {
            let input = f.t_named(sub);
            let container = f.t_named(sup);
            assert!(!atomic_is_contained(f, input, container, &symbols), "{sub} should NOT be a subtype of {sup}");
        }
    });
}

#[test]
fn symbol_table_transitive_closure() {
    fixture(|f| {
        let symbols = animal_hierarchy(f.arena);
        let cocker = f.name("Cocker");
        let animal = f.name("Animal");
        let mammal = f.name("Mammal");
        let sloth = f.name("Sloth");
        let cat = f.name("Cat");
        assert!(symbols.descends_from(cocker.id, animal.id));
        assert!(symbols.descends_from(cocker.id, mammal.id));
        assert!(symbols.descends_from(sloth.id, animal.id));
        assert!(symbols.descends_from(mammal.id, animal.id));
        assert!(!symbols.descends_from(animal.id, mammal.id));
        assert!(!symbols.descends_from(cocker.id, cat.id));
    });
}
