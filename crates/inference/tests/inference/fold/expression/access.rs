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
