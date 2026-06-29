use mago_hir::ir::statement::StatementKind;

use crate::harness::*;

mod annotation;
mod branch;
mod namespace;

test_inference! {
    name = statement_after_exit_is_unreachable,
    code = "<?php exit(1); $a = 1;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "$a = 1 follows a diverging exit(1)");
    }
}

test_inference! {
    name = return_with_diverging_value_diverges,
    code = "<?php return exit(0);",
    expect = |ir| {
        assert_eq!(get_last_statement(ir).meta.exit, ControlFlow::Diverge, "the value exits before the return runs");
    }
}

test_inference! {
    name = assignment_with_diverging_value_makes_continuation_unreachable,
    code = "<?php $a = exit(1); return $a;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "the return follows a diverging assignment");
    }
}

test_inference! {
    name = code_after_fully_returning_if_is_unreachable,
    code = "<?php /** @var bool */ $c = true; if ($c) { return 1; } else { return 2; } $x;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "both branches return, so $x is unreachable");
    }
}

test_inference! {
    name = statement_inside_dead_branch_is_unreachable,
    code = "<?php if (false) { $x = 1; }",
    expect = |ir| {
        let Some(if_statement) = ir.statements.iter().find(|statement| matches!(statement.kind, StatementKind::If(_)))
        else {
            panic!("expected an if statement")
        };
        let StatementKind::If(conditional) = if_statement.kind else { panic!("expected an if statement") };
        let any_then_reachable = match conditional.then.kind {
            StatementKind::Sequence(statements) => statements.iter().any(|statement| statement.meta.reachable),
            _ => conditional.then.meta.reachable,
        };
        assert!(!any_then_reachable, "the then-branch of `if (false)` is dead");
    }
}

test_inference! {
    name = namespace_reachability_threads_through_divergence,
    code = "<?php namespace A { exit(0); $b = 1; } namespace B { $c = 2; }",
    expect = |ir| {
        let namespaces: Vec<_> =
            ir.statements.iter().filter(|statement| matches!(statement.kind, StatementKind::Namespace(_))).collect();
        assert_eq!(namespaces.len(), 2);
        assert!(namespaces[0].meta.reachable, "namespace A is reachable");
        assert_eq!(namespaces[0].meta.exit, ControlFlow::Diverge, "namespace A diverges");
        assert!(!namespaces[1].meta.reachable, "namespace B follows a diverging namespace");

        let StatementKind::Namespace(b) = namespaces[1].kind else { panic!("expected a namespace") };
        let any_inner_reachable = match b.statement.kind {
            StatementKind::Sequence(statements) => statements.iter().any(|statement| statement.meta.reachable),
            other => matches!(other, StatementKind::Expression(_)) && b.statement.meta.reachable,
        };
        assert!(!any_inner_reachable, "$c = 2 inside the unreachable namespace B is unreachable too");
    }
}

test_inference! {
    name = navigates_individual_statements,
    code = "<?php $a = 1; $a;",
    expect = |ir| {
        let last = get_last_statement(ir);
        assert!(last.meta.reachable);
        assert_eq!(last.meta.exit, ControlFlow::Fallthrough);
        assert_eq!(expression_of(last).meta.to_string(), "int(1)");
        assert_eq!(expression_of(get_nth_statement(ir, 1)).meta.to_string(), "int(1)");
    }
}

test_inference! {
    name = statements_after_return_are_unreachable,
    code = "<?php $a = 1; return; $b = 2;",
    expect = |ir| {
        let flows: Vec<_> = ir.statements.iter().map(|statement| (statement.meta.reachable, statement.meta.exit)).collect();
        assert_eq!(
            flows,
            vec![
                (true, ControlFlow::Fallthrough),
                (true, ControlFlow::Fallthrough),
                (true, ControlFlow::Return),
                (false, ControlFlow::Fallthrough),
            ],
        );
    }
}

test_inference! {
    name = throw_terminates_flow,
    code = "<?php throw $e; $x;",
    expect = |ir| {
        let flows: Vec<_> = ir.statements.iter().map(|statement| (statement.meta.reachable, statement.meta.exit)).collect();
        assert_eq!(flows, vec![(true, ControlFlow::Fallthrough), (true, ControlFlow::Diverge), (false, ControlFlow::Fallthrough)]);
    }
}

test_inference! {
    name = exit_terminates_flow,
    code = "<?php exit(); $x;",
    expect = |ir| {
        let flows: Vec<_> = ir.statements.iter().map(|statement| (statement.meta.reachable, statement.meta.exit)).collect();
        assert_eq!(flows, vec![(true, ControlFlow::Fallthrough), (true, ControlFlow::Diverge), (false, ControlFlow::Fallthrough)]);
    }
}

test_inference! {
    name = straight_line_code_falls_through,
    code = "<?php $a = 1; $b = 2;",
    expect = |ir| {
        let flows: Vec<_> = ir.statements.iter().map(|statement| (statement.meta.reachable, statement.meta.exit)).collect();
        assert_eq!(flows, vec![(true, ControlFlow::Fallthrough), (true, ControlFlow::Fallthrough), (true, ControlFlow::Fallthrough)]);
    }
}

test_inference! {
    name = echo_infers_its_operands,
    code = "<?php echo 1 + 2;",
    expect = |ir| {
        let StatementKind::Echo(expressions) = get_last_statement(ir).kind else { panic!("expected an echo") };
        assert_eq!(expressions[0].meta.to_string(), "int(3)", "the echo operand is inferred");
    }
}

test_inference! {
    name = static_variable_binds_its_initializer,
    cases = { "<?php static $x = 5; $x;" => "int(5)" }
}

test_inference! {
    name = global_variable_is_mixed_without_annotation,
    cases = { "<?php global $y; $y;" => "mixed" }
}

test_inference! {
    name = unset_forgets_the_variable,
    cases = { "<?php $x = 5; unset($x); $x;" => "mixed" }
}

test_inference! {
    name = halt_compiler_diverges,
    code = "<?php $a = 1; __halt_compiler();",
    expect = |ir| {
        assert_eq!(get_last_statement(ir).meta.exit, ControlFlow::Diverge, "code after __halt_compiler() does not run");
    }
}
