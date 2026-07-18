test_inference! {
    name = clone_keeps_objects_and_rejects_scalars,
    cases = {
        "<?php clone $x;" => "mixed",
        "<?php $x = 1; clone $x;" => "never",
    }
}
