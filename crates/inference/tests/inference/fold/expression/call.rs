use indoc::indoc;

test_inference! {
    name = resolves_declared_return_type,
    def = "<?php function answer(): int { return 42; }",
    cases = { "<?php answer();" => "int" }
}

test_inference! {
    name = unknown_function_is_mixed,
    cases = { "<?php undefined_function();" => "mixed" }
}

test_inference! {
    name = instantiates_template_return_from_arguments,
    def = indoc! {"
        <?php

        /**
         * @template T
         * @template B
         * @param T $a
         * @param B $b
         * @return list{T, B}
         */
        function foo($a, $b): array { return [$a, $b]; }
    "},
    cases = { "<?php foo(1, 2);" => "list{0: int(1), 1: int(2)}" }
}

test_inference! {
    name = destructures_chained_template_call,
    def = indoc! {"
        <?php

        /**
         * @template T
         * @template B
         * @param T $a
         * @param B $b
         * @return list{T, B}
         */
        function foo($a, $b): array { return [$a, $b]; }
    "},
    cases = {
        "<?php [$a, $b] = $c = foo(1, 2); $c;" => "list{0: int(1), 1: int(2)}",
        "<?php [$a, $b] = $c = foo(1, 2); $a;" => "int(1)",
        "<?php [$a, $b] = $c = foo(1, 2); $b;" => "int(2)",
    }
}

test_inference! {
    name = single_template_identity_return,
    def = indoc! {"
        <?php
        /**
         * @template T
         * @param T $value
         * @return T
         */
        function identity($value) { return $value; }
    "},
    cases = {
        "<?php identity(7);" => "int(7)",
        "<?php identity('hi');" => "string('hi')",
    }
}
