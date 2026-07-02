test_inference! {
    name = yield_is_mixed_for_now,
    cases = {
        "<?php yield;" => "mixed",
        "<?php yield 1;" => "mixed",
        "<?php yield $k => $v;" => "mixed",
        "<?php yield from $gen;" => "mixed",
    }
}
