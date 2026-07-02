test_inference! {
    name = goto_then_label_falls_through,
    cases = { "<?php goto skip; skip: $a = 1; $a;" => "int(1)" }
}
