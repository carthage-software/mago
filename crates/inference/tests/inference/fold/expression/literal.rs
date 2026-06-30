use crate::harness::*;

test_inference! {
    name = infers_literals,
    cases = {
        "<?php 1;" => "int(1)",
        "<?php 1.5;" => "float(1.5)",
        "<?php true;" => "true",
        "<?php null;" => "null",
        "<?php 'foo';" => "string('foo')",
        "<?php '';" => "string('')",
    }
}

test_inference! {
    name = infers_assignment_value_type,
    cases = {
        "<?php $a = 1;" => "int(1)",
        "<?php $a = 'hi';" => "string('hi')",
    }
}

test_inference! {
    name = assignment_binds_target_variable_type,
    code = "<?php $a = 1;",
    expect = |ir| {
        let expression = get_last_expression(ir);
        let ExpressionKind::Assignment(assignment) = expression.kind else { panic!("expected an assignment") };
        assert_eq!(assignment.left.meta.to_string(), "int(1)");
    }
}
