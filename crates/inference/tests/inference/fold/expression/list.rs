test_inference! {
    name = list_in_value_position_is_never,
    cases = {
        "<?php list($a, $b);" => "never",
    }
}
