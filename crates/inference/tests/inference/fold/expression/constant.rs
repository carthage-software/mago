test_inference! {
    name = resolves_global_int_constant,
    def = "<?php const FOO = 1;",
    cases = {
        "<?php FOO;" => "int(1)",
        "<?php UNDEFINED;" => "mixed",
    }
}

test_inference! {
    name = resolves_global_string_constant,
    def = "<?php const NAME = 'hi';",
    cases = { "<?php NAME;" => "string('hi')" }
}

test_inference! {
    name = resolves_global_bool_constant,
    def = "<?php const FLAG = true;",
    cases = { "<?php FLAG;" => "true" }
}

test_inference! {
    name = float_constant_widens_to_float,
    def = "<?php const PI = 3.14;",
    cases = { "<?php PI;" => "float" }
}

test_inference! {
    name = fully_qualified_reference_resolves,
    def = "<?php const FOO = 7;",
    cases = { "<?php \\FOO;" => "int(7)" }
}

test_inference! {
    name = declared_type_wins_over_value,
    def = "<?php /** @var int */ const FOO = 1;",
    cases = { "<?php FOO;" => "int" }
}
