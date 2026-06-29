use crate::harness::*;

mod branch;
mod namespace;

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
