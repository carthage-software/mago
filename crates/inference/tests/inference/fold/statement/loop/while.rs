use mago_hir::ir::statement::StatementKind;

use crate::harness::*;

test_inference! {
    name = while_true_makes_following_code_unreachable,
    code = "<?php while (true) { $x = 1; } $y = 2;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "an infinite while with no break never falls through");
    }
}

test_inference! {
    name = while_with_break_falls_through,
    code = "<?php while (true) { break; } $y = 2;",
    expect = |ir| {
        assert!(get_last_statement(ir).meta.reachable, "a break lets control reach the code after the loop");
    }
}

test_inference! {
    name = while_negates_condition_after_the_loop,
    cases = {
        "<?php /** @var int|null */ $a = null; while ($a === null) { /** @var int */ $a = 0; } $a;" => "int",
    }
}

test_inference! {
    name = continue_keeps_the_loop_analyzable_and_escapable,
    code = "<?php /** @var bool */ $c = true; while ($c) { continue; } $x = 1;",
    expect = |ir| {
        assert!(get_last_statement(ir).meta.reachable, "a continue loops back; the condition can still end the loop");
    }
}

test_inference! {
    name = break_environment_reaches_the_code_after_the_loop,
    cases = {
        "<?php $x = 'a'; while (true) { $x = 1; break; } $x;" => "int(1)",
    }
}

test_inference! {
    name = a_while_loop_is_a_while_node,
    code = "<?php $i = 0; while ($i < 10) { $i = $i + 1; }",
    expect = |ir| {
        assert!(
            ir.statements.iter().any(|statement| matches!(statement.kind, StatementKind::While(_))),
            "the while statement survives inference",
        );
    }
}
