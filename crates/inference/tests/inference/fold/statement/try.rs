use crate::harness::*;

test_inference! {
    name = try_catch_falls_through_to_following_code,
    cases = { "<?php try { $a = 1; } catch (\\Exception $e) { $a = 2; } $b = 3; $b;" => "int(3)" }
}

test_inference! {
    name = finally_return_makes_continuation_unreachable,
    code = "<?php try { $a = 1; } finally { return 1; } $x;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "finally returns, so code after the try is unreachable");
    }
}
