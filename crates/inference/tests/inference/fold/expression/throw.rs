test_inference! {
    name = throw_is_never,
    cases = {
        "<?php throw $e;" => "never",
    }
}
