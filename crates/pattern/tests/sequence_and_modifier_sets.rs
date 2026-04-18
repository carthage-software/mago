use mago_pattern::test_matches;
use mago_pattern::test_no_match;

mod sequence_metavariables {
    use super::*;

    test_matches! {
        name = variadic_matches_zero_args,
        pattern = "foo(^...args)",
        code = "<?php foo();",
        expect = 1,
    }

    test_matches! {
        name = variadic_matches_single_arg,
        pattern = "foo(^...args)",
        code = "<?php foo($a);",
        expect = 1,
    }

    test_matches! {
        name = variadic_matches_many_args,
        pattern = "foo(^...args)",
        code = "<?php foo($a, $b, $c, 1, 'x');",
        expect = 1,
    }

    test_matches! {
        name = variadic_matches_many_calls_with_different_arities,
        pattern = "foo(^...args)",
        code = "<?php foo(); foo($a); foo($a, $b); foo($a, $b, $c);",
        expect = 4,
    }

    test_matches! {
        name = variadic_with_required_first,
        pattern = "foo(^first, ^...rest)",
        code = "<?php foo($a, $b, $c);",
        expect = 1,
    }

    test_no_match! {
        name = variadic_with_required_first_no_match_on_empty,
        pattern = "foo(^first, ^...rest)",
        code = "<?php foo();",
    }

    test_matches! {
        name = anonymous_dots_matches_everything,
        pattern = "foo(^...)",
        code = "<?php foo(1, 2, 3);",
        expect = 1,
    }
}

mod modifier_sets {
    use super::*;

    test_matches! {
        name = readonly_public_matches_either_order,
        pattern = "public readonly string $^name;",
        code = "<?php class Foo {
            public readonly string $a;
            readonly public string $b;
        }",
        expect = 2,
    }

    test_matches! {
        name = subset_matches_with_extra_modifiers,
        pattern = "public string $^name;",
        code = "<?php class Foo {
            public static string $a;
            public readonly string $b;
        }",
        expect = 2,
    }

    test_matches! {
        name = zero_modifier_pattern_matches_any_property,
        pattern = "string $^name;",
        code = "<?php class Foo {
            public string $a;
            private readonly string $b;
        }",
        expect = 2,
    }

    test_no_match! {
        name = missing_modifier_fails,
        pattern = "public readonly string $^name;",
        code = "<?php class Foo { public string $a; }",
    }

    test_matches! {
        name = readonly_any_visibility,
        pattern = "readonly string $^name;",
        code = "<?php class Foo {
            public readonly string $a;
            private readonly string $b;
            protected readonly string $c;
        }",
        expect = 3,
    }
}
