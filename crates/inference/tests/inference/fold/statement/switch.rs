use mago_hir::ir::statement::StatementKind;
use mago_hir::ir::statement::SwitchCase;

use crate::harness::*;

test_inference! {
    name = narrows_subject_in_a_later_case_after_earlier_cases_break,
    code = "<?php /** @var 1|2|3 */ $a = 1; switch ($a) { case 1: break; case 2: break; case 3: if ($a === 1) { echo $a; } }",
    expect = |ir| {
        let Some(switch_statement) =
            ir.statements.iter().find(|statement| matches!(statement.kind, StatementKind::Switch(_)))
        else {
            panic!("expected a switch")
        };
        let StatementKind::Switch(switch) = switch_statement.kind else { unreachable!() };
        let Some(&SwitchCase::Expression(_, body)) = switch.cases.items.last() else {
            panic!("expected case 3 to carry a value")
        };
        let Some(if_statement) = first_if(body) else { panic!("case 3 contains an if") };
        let StatementKind::If(conditional) = if_statement.kind else { unreachable!() };

        assert_eq!(
            conditional.condition.meta.to_string(),
            "false",
            "case 1 and case 2 broke out, so in case 3 $a is int(3), making `$a === 1` statically false",
        );
        assert!(!conditional.then.meta.reachable, "the then-branch of an always-false condition is unreachable");
    }
}

fn first_if<'arena>(statement: &'arena TypedStatement<'arena>) -> Option<&'arena TypedStatement<'arena>> {
    match statement.kind {
        StatementKind::If(_) => Some(statement),
        StatementKind::Sequence(statements) => statements.iter().find_map(first_if),
        _ => None,
    }
}

test_inference! {
    name = switch_with_a_break_falls_through,
    code = "<?php $x = 0; switch ($x) { case 1: break; } $y = 1;",
    expect = |ir| {
        assert!(get_last_statement(ir).meta.reachable, "control continues after a switch that can fall through");
    }
}

test_inference! {
    name = exhaustive_returning_switch_diverges,
    code = "<?php $x = 0; switch ($x) { case 1: return 1; default: return 2; } $y = 1;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "every case returns and a default covers the rest");
    }
}

test_inference! {
    name = switch_without_default_falls_through,
    code = "<?php $x = 0; switch ($x) { case 1: return 1; } $y = 1;",
    expect = |ir| {
        assert!(get_last_statement(ir).meta.reachable, "without a default the subject may match nothing");
    }
}
