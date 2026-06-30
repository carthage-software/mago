test_inference! {
    name = global_variable_is_mixed_without_annotation,
    cases = { "<?php global $y; $y;" => "mixed" }
}
