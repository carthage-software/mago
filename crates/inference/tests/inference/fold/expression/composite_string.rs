test_inference! {
    name = composite_string,
    cases = {
        "<?php $x = 1; \"n$x\";" => "string('n1')",
        "<?php $x = 'b'; \"a{$x}c\";" => "string('abc')",
        "<?php $x = 1.5; \"v$x\";" => "non-empty-string",
        "<?php \"$undefined\";" => "string",
    }
}

test_inference! {
    name = shell_execute,
    cases = {
        "<?php `echo hi`;" => "false|null|string",
    }
}
