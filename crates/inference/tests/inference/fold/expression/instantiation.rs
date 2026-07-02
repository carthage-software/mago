use indoc::indoc;

/// `Dog extends Animal`, `use`s a trait, and adds its own members — so a `new Dog`
/// instance must resolve own, inherited, and trait members.
const HIERARCHY: &str = indoc! {"
    <?php
    class Animal {
        public string $name = '';
        public function speak(): string { return '...'; }
    }

    trait Loud {
        public function shout(): string { return 'HEY'; }
    }

    class Dog extends Animal {
        use Loud;
        public int $legs = 4;
        public function bark(): int { return 1; }
    }
"};

test_inference! {
    name = new_yields_an_instance_of_the_class,
    def = HIERARCHY,
    cases = { "<?php new Dog();" => "Dog" }
}

test_inference! {
    name = calls_an_own_method_on_a_new_instance,
    def = HIERARCHY,
    cases = { "<?php $d = new Dog(); $d->bark();" => "int" }
}

test_inference! {
    name = reads_an_own_property_on_a_new_instance,
    def = HIERARCHY,
    cases = { "<?php $d = new Dog(); $d->legs;" => "int" }
}

test_inference! {
    name = calls_an_inherited_method_on_a_new_instance,
    def = HIERARCHY,
    cases = { "<?php $d = new Dog(); $d->speak();" => "string" }
}

test_inference! {
    name = reads_an_inherited_property_on_a_new_instance,
    def = HIERARCHY,
    cases = { "<?php $d = new Dog(); $d->name;" => "string" }
}

test_inference! {
    name = calls_a_trait_method_on_a_new_instance,
    def = HIERARCHY,
    cases = { "<?php $d = new Dog(); $d->shout();" => "string" }
}

test_inference! {
    name = an_unknown_class_still_yields_its_named_object,
    cases = { "<?php new Whatever();" => "Whatever" }
}

test_inference! {
    name = a_dynamic_class_instantiation_is_mixed,
    cases = { "<?php /** @var class-string */ $c = ''; new $c();" => "mixed" }
}
