use crate::harness::*;

test_inference! {
    name = do_while_runs_the_body_at_least_once,
    cases = {
        "<?php $x = 'init'; do { $x = 5; } while (false); $x;" => "int(5)",
    }
}

test_inference! {
    name = do_while_true_makes_following_code_unreachable,
    code = "<?php do { $x = 1; } while (true); $y = 2;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "an infinite do-while with no break never falls through");
    }
}
