test_inference! {
    name = join_unions_both_branch_assignments,
    cases = {
        "<?php /** @var bool */ $c = true; if ($c) { $x = 1; } else { $x = 'a'; } $x;" => "int(1)|string('a')",
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
