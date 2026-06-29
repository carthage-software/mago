test_inference! {
    name = static_variable_binds_its_initializer,
    cases = { "<?php static $x = 5; $x;" => "int(5)" }
}
