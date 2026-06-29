test_inference! {
    name = resolves_literal_subject_exactly,
    cases = {
        "<?php match (1) { 1 => 'a', 2 => 'b' };" => "string('a')",
        "<?php match (3) { 1, 2 => 'a', default => 'd' };" => "string('d')",
        "<?php match (false) { true => 1 };" => "never",
    }
}

test_inference! {
    name = unions_results_of_reachable_arms,
    cases = {
        "<?php /** @var 1|2|3 */ $a = 1; match ($a) { 1 => 'one', 2 => 'two', 3 => 'three' };"
            => "string('one')|string('three')|string('two')",
    }
}

test_inference! {
    name = narrows_subject_variable_inside_arm,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; match ($a) { 1 => $a, 2 => $a + 1 };" => "int(1)|int(3)",
    }
}

test_inference! {
    name = narrows_array_element_subject_inside_arm,
    cases = {
        "<?php /** @var array{0: 1|2} */ $a = []; match ($a[0]) { 1 => $a[0], 2 => $a[0] + 1 };" => "int(1)|int(3)",
    }
}

test_inference! {
    name = unreachable_arm_is_excluded_from_union,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; match ($a) { 1 => 'one', 2 => 'two', 3 => 'never' };"
            => "string('one')|string('two')",
    }
}

test_inference! {
    name = exhausted_subject_makes_default_unreachable,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; match ($a) { 1 => 'one', 2 => 'two', default => 'd' };"
            => "string('one')|string('two')",
    }
}

test_inference! {
    name = non_exhaustive_without_default_drops_unmatched_path,
    cases = {
        "<?php /** @var int */ $a = 1; match ($a) { 1 => 'one' };" => "string('one')",
    }
}

test_inference! {
    name = default_handles_the_unmatched_remainder,
    cases = {
        "<?php /** @var int */ $a = 1; match ($a) { 1 => 'one', default => 'rest' };"
            => "string('one')|string('rest')",
    }
}

test_inference! {
    name = arm_assignment_joins_after_match,
    cases = {
        "<?php /** @var 1|2 */ $a = 1; match ($a) { 1 => $b = 'x', 2 => $b = 'y' }; $b;"
            => "string('x')|string('y')",
    }
}
