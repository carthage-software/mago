use mago_hir::ir::statement::StatementKind;

use crate::harness::*;

test_inference! {
    name = binds_the_value_to_the_element_type,
    code = "<?php /** @var list<int> */ $a = []; foreach ($a as $v) { $v; }",
    expect = |ir| {
        let Some(statement) = ir.statements.iter().find(|statement| matches!(statement.kind, StatementKind::Foreach(_)))
        else {
            panic!("expected a foreach statement")
        };
        let StatementKind::Foreach(foreach) = statement.kind else { unreachable!() };
        assert_eq!(foreach.value.meta.to_string(), "int", "the value variable takes the list element type");
    }
}

test_inference! {
    name = value_from_a_known_shape_unions_its_entries,
    code = "<?php $last = 0; foreach (['x' => 1, 'y' => 2] as $v) { $last = $v; }",
    expect = |ir| {
        let Some(statement) = ir.statements.iter().find(|statement| matches!(statement.kind, StatementKind::Foreach(_)))
        else {
            panic!("expected a foreach statement")
        };
        let StatementKind::Foreach(foreach) = statement.kind else { unreachable!() };
        assert_eq!(foreach.value.meta.to_string(), "int(1)|int(2)", "the value unions the shape's entry types");
    }
}
