test_inference! {
    name = anonymous_var_overrides_assigned_value,
    cases = {
        "<?php $x = `ls`; $x;" => "false|null|string",
        "<?php /** @var string */ $x = `ls`; $x;" => "string",
        "<?php /** @var int */ $x = 'hello'; $x;" => "int",
        "<?php /** @var non-empty-string */ $x = 1; $x;" => "non-empty-string",
    }
}

test_inference! {
    name = named_var_pins_the_target_variable,
    cases = {
        "<?php /** @var string $bar */ $bar = 1; $bar;" => "string",
    }
}

test_inference! {
    name = annotated_assignment_evaluates_to_annotated_type,
    cases = {
        "<?php /** @var string */ $b = 1;" => "string",
        "<?php /** @var string $b */ $b = 1;" => "string",
        "<?php /** @var string $b */ $b = 1; $c = $b; $c;" => "string",
    }
}
