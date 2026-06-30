test_inference! {
    name = namespaced_constant_resolves,
    def = "<?php namespace App; const FOO = 1;",
    cases = {
        "<?php namespace App; FOO;" => "int(1)",
    }
}

test_inference! {
    name = global_constant_resolution_across_namespaces,
    def = "<?php const FOO = 5;",
    cases = {
        "<?php namespace App; FOO;" => "int(5)",
        "<?php namespace App; \\FOO;" => "int(5)",
        "<?php namespace App; Sub\\FOO;" => "mixed",
    }
}
