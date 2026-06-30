use indoc::indoc;

use crate::harness::*;

test_inference! {
    name = compound_assignment_applies_the_binary_operation,
    cases = {
        "<?php $a = 1; $a += 2; $a;" => "int(3)",
        "<?php $a = 5; $a -= 2; $a;" => "int(3)",
        "<?php $s = 'a'; $s .= 'b'; $s;" => "string('ab')",
    }
}

test_inference! {
    name = null_coalescing_assignment_combines_the_fallback,
    cases = { "<?php /** @var int|null */ $a = null; $a ??= 7; $a;" => "int|int(7)" }
}

test_inference! {
    name = appending_then_keying_builds_a_keyed_array,
    cases = { "<?php $a = []; $a[] = 1; $a['b'] = 2; $a;" => "array{0: int(1), 'b': int(2)}" }
}

test_inference! {
    name = appending_to_a_list_grows_the_list,
    cases = { "<?php $a = [1, 2, 3]; $a[] = [1]; $a;" => "list{0: int(1), 1: int(2), 2: int(3), 3: list{0: int(1)}}" }
}

test_inference! {
    name = appending_to_an_empty_array_yields_a_list,
    cases = { "<?php $a = []; $a[] = 1; $a[] = 2; $a;" => "list{0: int(1), 1: int(2)}" }
}

test_inference! {
    name = a_key_write_replaces_an_existing_entry,
    cases = { "<?php $a = ['x' => 1]; $a['x'] = 'str'; $a;" => "array{'x': string('str')}" }
}

test_inference! {
    name = an_element_write_is_visible_to_a_later_read,
    cases = {
        "<?php $a = []; $a[0] = 5; $a[0];" => "int(5)",
        "<?php $a = []; $a['k'] = 'v'; $a['k'];" => "string('v')",
    }
}

test_inference! {
    name = a_dynamic_key_write_keeps_the_written_key_type,
    cases = { "<?php /** @var string */ $k = ''; $a = []; $a[$k] = 1; $a;" => "array<string, int(1)>" }
}

test_inference! {
    name = a_dynamic_int_key_widens_a_string_keyed_array_to_array_key,
    cases = {
        indoc! {"
            <?php
            /** @var array<string, int> */
            $a = [];
            /** @var int */
            $b = 0;
            $c = $a;
            $c[$b] = 1;
            $c;
        "} => "array<array-key, int>",
    }
}

test_inference! {
    name = a_cast_string_key_keeps_a_string_keyed_array,
    cases = {
        indoc! {"
            <?php
            /** @var array<string, int> */
            $a = [];
            /** @var int */
            $b = 0;
            $c = $a;
            $c[(string) $b] = 1;
            $c;
        "} => "array<string, int>",
    }
}

test_inference! {
    name = a_dynamic_key_write_to_a_shape_keeps_known_entries_and_adds_a_rest,
    cases = {
        indoc! {"
            <?php
            /** @var int */
            $b = 0;
            $d = array('a' => 1, 'b' => 2);
            $d[$b] = 'hello';
            $d;
        "} => "array{'a': int(1), 'b': int(2), ...<int, string('hello')>}",
    }
}

test_inference! {
    name = repeated_dynamic_key_writes_union_into_the_rest_value,
    cases = {
        indoc! {"
            <?php
            /** @var int */
            $b = 0;
            $d = array('a' => 1, 'b' => 2);
            $d[$b] = 'hello';
            $d[$b] = [1];
            $d;
        "} => "array{'a': int(1), 'b': int(2), ...<int, list{0: int(1)}|string('hello')>}",
    }
}
