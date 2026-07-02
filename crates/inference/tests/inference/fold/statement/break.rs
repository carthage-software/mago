use crate::harness::*;

test_inference! {
    name = literal_break_exits_the_switch,
    code = "<?php $x = 0; switch ($x) { case 1: break; default: return 1; } $y = 1;",
    expect = |ir| {
        assert!(get_last_statement(ir).meta.reachable, "a literal break exits the switch");
    }
}

test_inference! {
    name = non_literal_break_level_is_a_fatal_divergence,
    code = "<?php $x = 0; switch ($x) { case 1: break $x; default: return 1; } $y = 1;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "break with a non-literal level is a php fatal");
    }
}
