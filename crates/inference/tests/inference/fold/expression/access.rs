test_inference! {
    name = reads_static_property_type,
    def = "<?php class C { public static int $count = 0; }",
    cases = { "<?php C::$count;" => "int" }
}

test_inference! {
    name = reads_instance_property_type,
    def = "<?php class C { public int $count = 0; }",
    cases = { "<?php /** @var C */ $c = null; $c->count;" => "int" }
}

test_inference! {
    name = a_static_property_read_as_an_instance_is_mixed,
    def = "<?php class C { public static int $count = 0; }",
    cases = { "<?php /** @var C */ $c = null; $c->count;" => "mixed" }
}

test_inference! {
    name = a_property_write_is_visible_to_a_later_read,
    def = "<?php class C { public int $count = 0; }",
    cases = { "<?php /** @var C */ $c = null; $c->count = 5; $c->count;" => "int(5)" }
}

test_inference! {
    name = a_property_write_yields_the_assigned_value,
    def = "<?php class C { public static int $total = 0; }",
    cases = { "<?php C::$total = 7;" => "int(7)" }
}

test_inference! {
    name = nullsafe_property_read_on_nullable_unions_null,
    def = "<?php class C { public int $count = 0; }",
    cases = {
        "<?php /** @var C|null */ $c = null; $c?->count;" => "int|null",
        "<?php /** @var C */ $c = null; $c?->count;" => "int",
    }
}

test_inference! {
    name = unknown_property_or_non_object_receiver_is_mixed,
    def = "<?php class C { public int $count = 0; }",
    cases = {
        "<?php /** @var C */ $c = null; $c->missing;" => "mixed",
        "<?php /** @var int */ $x = 0; $x->count;" => "mixed",
    }
}

test_inference! {
    name = a_non_static_property_read_statically_is_mixed,
    def = "<?php class C { public int $x = 0; }",
    cases = { "<?php C::$x;" => "mixed" }
}

test_inference! {
    name = reads_class_constant_type,
    def = "<?php class C { const LIMIT = 10; }",
    cases = { "<?php C::LIMIT;" => "int(10)" }
}

test_inference! {
    name = class_keyword_is_a_literal_class_string,
    def = "<?php class C {}",
    cases = { "<?php C::class;" => "class-string('C')" }
}

test_inference! {
    name = reads_enum_case_as_its_singleton,
    def = "<?php enum Status: string { case Active = 'a'; case Inactive = 'b'; }",
    cases = {
        "<?php Status::Active;" => "enum(Status::Active)",
        "<?php Status::Inactive;" => "enum(Status::Inactive)",
    }
}

test_inference! {
    name = enum_constant_resolves_separately_from_its_cases,
    def = "<?php enum E: int { case A = 1; const MAX = 100; }",
    cases = {
        "<?php E::A;" => "enum(E::A)",
        "<?php E::MAX;" => "int(100)",
    }
}

test_inference! {
    name = unknown_class_member_is_mixed,
    cases = {
        "<?php Unknown::FOO;" => "mixed",
        "<?php Unknown::$bar;" => "mixed",
    }
}

test_inference! {
    name = reads_array_element_by_known_key,
    cases = {
        "<?php $a = ['a' => 1, 'b' => 2]; $a['a'];" => "int(1)",
        "<?php $a = ['a' => 1, 'b' => 2]; $a['b'];" => "int(2)",
    }
}

test_inference! {
    name = reads_a_missing_key_as_null,
    cases = { "<?php $a = ['a' => 1]; $a['z'];" => "null" }
}

test_inference! {
    name = reads_dynamic_key_as_union_of_values_and_null,
    cases = {
        "<?php /** @var array{1: int, 2: string} */ $b = []; /** @var int */ $k = 1; $b[$k];" => "int|null|string",
    }
}

test_inference! {
    name = reads_open_shape_filtered_by_key_kind,
    cases = {
        "<?php /** @var array{1: int, 2: string, ...<array-key, float>} */ $b = []; /** @var int */ $k = 1; $b[$k];" => "float|int|null|string",
        "<?php /** @var array{1: int, 2: string, ...<array-key, float>} */ $b = []; /** @var string */ $s = ''; $b[$s];" => "float|null",
    }
}

test_inference! {
    name = unset_removes_an_array_element,
    cases = {
        "<?php $a = ['a' => 1, 'b' => 2]; unset($a['a']); $a['a'];" => "null",
        "<?php $a = ['a' => 1, 'b' => 2]; unset($a['a']); $a['b'];" => "int(2)",
    }
}

test_inference! {
    name = reads_an_element_from_a_list,
    cases = {
        "<?php /** @var list<int> */ $a = []; $a[0];" => "int|null",
        "<?php /** @var non-empty-list<string> */ $a = ['x']; $a[0];" => "string",
    }
}
