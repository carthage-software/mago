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
