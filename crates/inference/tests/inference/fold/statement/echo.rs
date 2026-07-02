use mago_hir::ir::statement::StatementKind;

use crate::harness::*;

test_inference! {
    name = echo_infers_its_operands,
    code = "<?php echo 1 + 2;",
    expect = |ir| {
        let StatementKind::Echo(expressions) = get_last_statement(ir).kind else { panic!("expected an echo") };
        assert_eq!(expressions[0].meta.to_string(), "int(3)", "the echo operand is inferred");
    }
}
