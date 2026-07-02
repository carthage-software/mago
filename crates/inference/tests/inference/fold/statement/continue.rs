use crate::harness::*;

test_inference! {
    name = continue_exits_the_switch_like_break,
    code = "<?php $x = 0; switch ($x) { case 1: continue; default: return 1; } $y = 1;",
    expect = |ir| {
        assert!(get_last_statement(ir).meta.reachable, "continue in a switch behaves like break");
    }
}
