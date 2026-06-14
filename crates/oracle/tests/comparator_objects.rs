mod common;

use common::*;

use mago_oracle::world::World as _;

fn animal_hierarchy<'arena>() -> MockWorld<'arena> {
    let mut world = MockWorld::from_edges(&[
        ("Mammal", "Animal"),
        ("Dog", "Mammal"),
        ("Cat", "Mammal"),
        ("Cocker", "Dog"),
        ("Poodle", "Dog"),
        ("Sloth", "Mammal"),
        ("Sloth", "Sleepable"),
    ]);
    world.declare("Standalone");
    world
}

fn enums_hierarchy<'arena>() -> MockWorld<'arena> {
    let mut world = MockWorld::new();
    world.declare("Status");
    world.declare("Color");
    world
}

#[test]
fn object_any_reflexive() {
    fixture(|f| {
        let object = f.t_object_any();
        assert!(atomic_is_contained(f, object, object, &empty_world()));
    });
}

#[test]
fn named_in_object_any() {
    fixture(|f| {
        let world = animal_hierarchy();
        let object = f.t_object_any();
        let dog = f.t_named("Dog");
        let cat = f.t_named("Cat");
        let standalone = f.t_named("Standalone");
        assert!(atomic_is_contained(f, dog, object, &world));
        assert!(atomic_is_contained(f, cat, object, &world));
        assert!(atomic_is_contained(f, standalone, object, &world));
    });
}

#[test]
fn object_any_not_in_named() {
    fixture(|f| {
        let world = animal_hierarchy();
        let object = f.t_object_any();
        let dog = f.t_named("Dog");
        assert!(!atomic_is_contained(f, object, dog, &world));
    });
}

#[test]
fn class_reflexive() {
    fixture(|f| {
        let world = animal_hierarchy();
        for n in ["Dog", "Cat", "Cocker", "Animal", "Standalone"] {
            let named = f.t_named(n);
            assert!(atomic_is_contained(f, named, named, &world));
        }
    });
}

#[test]
fn subclass_in_superclass() {
    fixture(|f| {
        let world = animal_hierarchy();
        let cocker = f.t_named("Cocker");
        let poodle = f.t_named("Poodle");
        let dog = f.t_named("Dog");
        assert!(atomic_is_contained(f, cocker, dog, &world));
        assert!(atomic_is_contained(f, poodle, dog, &world));
    });
}

#[test]
fn superclass_not_in_subclass() {
    fixture(|f| {
        let world = animal_hierarchy();
        let dog = f.t_named("Dog");
        let cocker = f.t_named("Cocker");
        assert!(!atomic_is_contained(f, dog, cocker, &world));
    });
}

#[test]
fn class_not_in_sibling() {
    fixture(|f| {
        let world = animal_hierarchy();
        let cocker = f.t_named("Cocker");
        let cat = f.t_named("Cat");
        let poodle = f.t_named("Poodle");
        let dog = f.t_named("Dog");
        assert!(!atomic_is_contained(f, cocker, cat, &world));
        assert!(!atomic_is_contained(f, cocker, poodle, &world));
        assert!(!atomic_is_contained(f, dog, cat, &world));
    });
}

#[test]
fn class_implements_interface() {
    fixture(|f| {
        let world = animal_hierarchy();
        let dog = f.t_named("Dog");
        let cat = f.t_named("Cat");
        let mammal = f.t_named("Mammal");
        assert!(atomic_is_contained(f, dog, mammal, &world));
        assert!(atomic_is_contained(f, cat, mammal, &world));
    });
}

#[test]
fn class_implements_inherited_interface() {
    fixture(|f| {
        let world = animal_hierarchy();
        let dog = f.t_named("Dog");
        let cocker = f.t_named("Cocker");
        let animal = f.t_named("Animal");
        assert!(atomic_is_contained(f, dog, animal, &world));
        assert!(atomic_is_contained(f, cocker, animal, &world));
    });
}

#[test]
fn interface_in_parent_interface() {
    fixture(|f| {
        let world = animal_hierarchy();
        let mammal = f.t_named("Mammal");
        let animal = f.t_named("Animal");
        assert!(atomic_is_contained(f, mammal, animal, &world));
    });
}

#[test]
fn class_implements_multiple_interfaces() {
    fixture(|f| {
        let world = animal_hierarchy();
        let sloth = f.t_named("Sloth");
        let mammal = f.t_named("Mammal");
        let sleepable = f.t_named("Sleepable");
        let animal = f.t_named("Animal");
        assert!(atomic_is_contained(f, sloth, mammal, &world));
        assert!(atomic_is_contained(f, sloth, sleepable, &world));
        assert!(atomic_is_contained(f, sloth, animal, &world));
    });
}

#[test]
fn class_does_not_implement_unrelated_interface() {
    fixture(|f| {
        let world = animal_hierarchy();
        let dog = f.t_named("Dog");
        let cat = f.t_named("Cat");
        let standalone = f.t_named("Standalone");
        let sleepable = f.t_named("Sleepable");
        let animal = f.t_named("Animal");
        assert!(!atomic_is_contained(f, dog, sleepable, &world));
        assert!(!atomic_is_contained(f, cat, sleepable, &world));
        assert!(!atomic_is_contained(f, standalone, animal, &world));
    });
}

#[test]
fn parent_interface_not_in_child() {
    fixture(|f| {
        let world = animal_hierarchy();
        let animal = f.t_named("Animal");
        let mammal = f.t_named("Mammal");
        assert!(!atomic_is_contained(f, animal, mammal, &world));
    });
}

#[test]
fn interface_not_in_class() {
    fixture(|f| {
        let world = animal_hierarchy();
        let mammal = f.t_named("Mammal");
        let animal = f.t_named("Animal");
        let dog = f.t_named("Dog");
        assert!(!atomic_is_contained(f, mammal, dog, &world));
        assert!(!atomic_is_contained(f, animal, dog, &world));
    });
}

#[test]
fn unrelated_classes_disjoint() {
    fixture(|f| {
        let world = animal_hierarchy();
        let dog = f.t_named("Dog");
        let standalone = f.t_named("Standalone");
        assert!(!atomic_is_contained(f, dog, standalone, &world));
        assert!(!atomic_is_contained(f, standalone, dog, &world));
    });
}

#[test]
fn class_not_in_object_when_not_a_subtype() {
    fixture(|f| {
        let world = animal_hierarchy();
        let cat = f.t_named("Cat");
        let cocker = f.t_named("Cocker");
        assert!(!atomic_is_contained(f, cat, cocker, &world));
    });
}

#[test]
fn enum_reflexive() {
    fixture(|f| {
        let world = enums_hierarchy();
        let status = f.t_enum("Status");
        let color = f.t_enum("Color");
        assert!(atomic_is_contained(f, status, status, &world));
        assert!(atomic_is_contained(f, color, color, &world));
    });
}

#[test]
fn enum_case_in_enum() {
    fixture(|f| {
        let world = enums_hierarchy();
        let status = f.t_enum("Status");
        let color = f.t_enum("Color");
        let active = f.t_enum_case("Status", "Active");
        let inactive = f.t_enum_case("Status", "Inactive");
        let red = f.t_enum_case("Color", "Red");
        assert!(atomic_is_contained(f, active, status, &world));
        assert!(atomic_is_contained(f, inactive, status, &world));
        assert!(atomic_is_contained(f, red, color, &world));
    });
}

#[test]
fn enum_not_in_enum_case() {
    fixture(|f| {
        let world = enums_hierarchy();
        let status = f.t_enum("Status");
        let active = f.t_enum_case("Status", "Active");
        assert!(!atomic_is_contained(f, status, active, &world));
    });
}

#[test]
fn enum_case_reflexive() {
    fixture(|f| {
        let world = enums_hierarchy();
        let active = f.t_enum_case("Status", "Active");
        assert!(atomic_is_contained(f, active, active, &world));
    });
}

#[test]
fn enum_cases_disjoint() {
    fixture(|f| {
        let world = enums_hierarchy();
        let active = f.t_enum_case("Status", "Active");
        let inactive = f.t_enum_case("Status", "Inactive");
        assert!(!atomic_is_contained(f, active, inactive, &world));
    });
}

#[test]
fn distinct_enums_disjoint() {
    fixture(|f| {
        let world = enums_hierarchy();
        let status = f.t_enum("Status");
        let color = f.t_enum("Color");
        let active = f.t_enum_case("Status", "Active");
        assert!(!atomic_is_contained(f, status, color, &world));
        assert!(!atomic_is_contained(f, active, color, &world));
    });
}

#[test]
fn class_not_in_int() {
    fixture(|f| {
        let world = animal_hierarchy();
        let dog = f.t_named("Dog");
        let int = f.t_int();
        assert!(!atomic_is_contained(f, dog, int, &world));
    });
}

#[test]
fn class_not_in_string() {
    fixture(|f| {
        let world = animal_hierarchy();
        let dog = f.t_named("Dog");
        let string = f.t_string();
        assert!(!atomic_is_contained(f, dog, string, &world));
    });
}

#[test]
fn class_in_mixed() {
    fixture(|f| {
        let world = animal_hierarchy();
        let dog = f.t_named("Dog");
        let object = f.t_object_any();
        let mixed = f.mixed();
        assert!(atomic_is_contained(f, dog, mixed, &world));
        assert!(atomic_is_contained(f, object, mixed, &world));
    });
}

#[test]
fn many_class_hierarchy_relations() {
    fixture(|f| {
        let world = animal_hierarchy();
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
            assert!(atomic_is_contained(f, input, container, &world), "{sub} should be a subtype of {sup}");
        }
    });
}

#[test]
fn many_class_hierarchy_non_relations() {
    fixture(|f| {
        let world = animal_hierarchy();
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
            assert!(!atomic_is_contained(f, input, container, &world), "{sub} should NOT be a subtype of {sup}");
        }
    });
}

#[test]
fn mock_world_transitive_closure() {
    fixture(|f| {
        let world = animal_hierarchy();
        let cocker = f.name("Cocker");
        let animal = f.name("Animal");
        let mammal = f.name("Mammal");
        let sloth = f.name("Sloth");
        let cat = f.name("Cat");
        assert!(world.descends_from(cocker, animal));
        assert!(world.descends_from(cocker, mammal));
        assert!(world.descends_from(sloth, animal));
        assert!(world.descends_from(mammal, animal));
        assert!(!world.descends_from(animal, mammal));
        assert!(!world.descends_from(cocker, cat));
    });
}
