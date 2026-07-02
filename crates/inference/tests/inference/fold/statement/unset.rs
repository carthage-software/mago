test_inference! {
    name = unset_forgets_the_variable,
    cases = { "<?php $x = 5; unset($x); $x;" => "mixed" }
}
