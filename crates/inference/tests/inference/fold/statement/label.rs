test_inference! {
    name = label_falls_through_to_next_statement,
    cases = { "<?php start: $a = 1; $a;" => "int(1)" }
}
