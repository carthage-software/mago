use mago_hir::ir::statement::StatementKind;

use crate::harness::*;

test_inference! {
    name = join_unions_both_branch_assignments,
    cases = {
        "<?php /** @var bool */ $c = true; if ($c) { $x = 1; } else { $x = 'a'; } $x;" => "int(1)|string('a')",
    }
}

test_inference! {
    name = narrows_array_element_places_in_each_branch,
    code = "<?php /** @var array{a: 1|2, b: 3|4} */ $a = []; if ($a['a'] === 2 && $a['b'] !== 4) { $a['a']; $a['b']; } else { $a['a']; $a['b']; }",
    expect = |ir| {
        let Some(branch) = ir.statements.iter().find(|statement| matches!(statement.kind, StatementKind::If(_)))
        else {
            panic!("expected an if statement")
        };
        let StatementKind::If(conditional) = branch.kind else { unreachable!() };

        assert_eq!(
            place_reads(conditional.then),
            vec!["int(2)".to_string(), "int(3)".to_string()],
            "in the then-branch the condition holds, so $a['a'] is int(2) and $a['b'] is int(3)",
        );

        let Some(otherwise) = conditional.r#else else { panic!("expected an else branch") };
        assert_eq!(
            place_reads(otherwise),
            vec!["int(1)|int(2)".to_string(), "int(3)|int(4)".to_string()],
            "the else-branch pins neither element, so each falls back to its shape union",
        );
    }
}

fn place_reads(statement: &TypedStatement<'_>) -> Vec<String> {
    let mut reads = Vec::new();
    collect_place_reads(statement, &mut reads);
    reads
}

fn collect_place_reads(statement: &TypedStatement<'_>, reads: &mut Vec<String>) {
    match statement.kind {
        StatementKind::Expression(expression) => reads.push(expression.meta.to_string()),
        StatementKind::Sequence(statements) => {
            for statement in statements {
                collect_place_reads(statement, reads);
            }
        }
        _ => {}
    }
}

test_inference! {
    name = narrows_then_branch_environment,
    cases = {
        "<?php /** @var string|null */ $a = null; if ($a === null) { $b = $a; } else { $b = $a; } $b;" => "null|string",
    }
}

test_inference! {
    name = returning_then_branch_leaves_else_type,
    cases = {
        "<?php /** @var int|null */ $a = null; if ($a === null) { return; } $a;" => "int",
    }
}

test_inference! {
    name = elseif_chain_narrows_each_arm,
    cases = {
        "<?php /** @var 1|2|3 */ $a = 1; if ($a === 1) { $b = 'one'; } elseif ($a === 2) { $b = 'two'; } else { $b = 'rest'; } $b;"
            => "string('one')|string('rest')|string('two')",
    }
}

test_inference! {
    name = no_else_unions_modified_with_passthrough,
    cases = {
        "<?php $x = 'a'; /** @var bool */ $c = true; if ($c) { $x = 1; } $x;" => "int(1)|string('a')",
    }
}

test_inference! {
    name = always_true_condition_takes_only_then,
    cases = {
        "<?php $x = 'a'; if (true) { $x = 1; } else { $x = 2; } $x;" => "int(1)",
    }
}

test_inference! {
    name = exhaustive_elseif_chain_proves_else_unreachable,
    cases = {
        "<?php /** @var bool */ $a = true; if ($a === true) { $x = 1; } elseif ($a === false) { $x = 2; } else { $x = 'dead'; } $x;"
            => "int(1)|int(2)",
    }
}
