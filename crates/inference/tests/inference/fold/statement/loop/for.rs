use crate::harness::*;

test_inference! {
    name = a_counter_widens_instead_of_enumerating_every_iteration,
    code = "<?php for ($i = 0; $i < 5; $i = $i + 1) {} $i;",
    expect = |ir| {
        // Without widening the counter would grow a fresh literal each pass
        // (`int(0)|int(1)|...`); widening collapses it to a small, bounded union.
        let counter = get_last_expression(ir);
        let atoms = counter.meta.atoms.len();
        assert!(atoms <= 4, "the counter stays a small union (got {atoms} atoms: {})", counter.meta);
    }
}

test_inference! {
    name = infinite_for_makes_following_code_unreachable,
    code = "<?php for (;;) { $x = 1; } $y = 2;",
    expect = |ir| {
        assert!(!get_last_statement(ir).meta.reachable, "for (;;) with no break never falls through");
    }
}

test_inference! {
    name = infinite_for_with_break_falls_through,
    code = "<?php for (;;) { break; } $y = 2;",
    expect = |ir| {
        assert!(get_last_statement(ir).meta.reachable, "a break ends the otherwise-infinite for");
    }
}
