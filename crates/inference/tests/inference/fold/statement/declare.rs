use crate::harness::*;

test_inference! {
    name = declare_continues_to_following_code,
    cases = { "<?php declare(strict_types=1); $a = 1; $a;" => "int(1)" }
}

test_inference! {
    name = strict_types_declare_after_a_statement_is_unreachable,
    code = "<?php $a = 1; declare(strict_types=1);",
    expect = |ir| {
        assert!(
            !get_last_statement(ir).meta.reachable,
            "strict_types must be the first statement, so a later one is a fatal error",
        );
    }
}

test_inference! {
    name = declare_after_divergence_is_unreachable,
    code = "<?php exit; declare(ticks=1);",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "a statement after a diverging exit is unreachable");
    }
}
