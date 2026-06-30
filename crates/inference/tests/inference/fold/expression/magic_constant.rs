test_inference! {
    name = magic_constants,
    cases = {
        "<?php __LINE__;" => "int(1)",
        "<?php\n\n__LINE__;" => "int(3)",
        "<?php __FILE__;" => "non-empty-literal-string",
        "<?php __DIR__;" => "non-empty-literal-string",
        "<?php __FUNCTION__;" => "string('')",
        "<?php __CLASS__;" => "string('')",
        "<?php __METHOD__;" => "string('')",
        "<?php __NAMESPACE__;" => "string('')",
    }
}

test_inference! {
    name = namespace_magic_constant_is_exact,
    cases = {
        "<?php namespace App; __NAMESPACE__;" => "string('App')",
        "<?php namespace App\\Model; __NAMESPACE__;" => "string('App\\Model')",
    }
}
