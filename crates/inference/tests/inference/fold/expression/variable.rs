test_inference! {
    name = variable_read_write,
    cases = {
        "<?php $a = 1; $a;" => "int(1)",
        "<?php $a = 1; $b = 123; $a + $b;" => "int(124)",
        "<?php $a = 'x'; $b = 'y'; $a . $b;" => "string('xy')",
        "<?php $a = 1; $a = 'two'; $a;" => "string('two')",
        "<?php $undefined;" => "mixed",
    }
}
