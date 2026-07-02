use indoc::indoc;

test_inference! {
    name = resolves_declared_return_type,
    def = "<?php function answer(): int { return 42; }",
    cases = { "<?php answer();" => "int" }
}

test_inference! {
    name = resolves_method_call_return_type,
    def = "<?php class C { public function size(): int { return 0; } }",
    cases = { "<?php /** @var C */ $c = null; $c->size();" => "int" }
}

test_inference! {
    name = resolves_static_method_call_return_type,
    def = "<?php class C { public static function make(): string { return ''; } }",
    cases = { "<?php C::make();" => "string" }
}

test_inference! {
    name = nullsafe_method_call_on_nullable_unions_null,
    def = "<?php class C { public function size(): int { return 0; } }",
    cases = {
        "<?php /** @var C|null */ $c = null; $c?->size();" => "int|null",
        "<?php /** @var C */ $c = null; $c?->size();" => "int",
    }
}

test_inference! {
    name = resolves_enum_instance_method_call,
    def = "<?php enum E { case A; public function label(): string { return ''; } }",
    cases = { "<?php /** @var E */ $e = E::A; $e->label();" => "string" }
}

test_inference! {
    name = unknown_method_or_non_object_receiver_is_mixed,
    def = "<?php class C { public function size(): int { return 0; } }",
    cases = {
        "<?php /** @var C */ $c = null; $c->missing();" => "mixed",
        "<?php /** @var int */ $x = 0; $x->size();" => "mixed",
    }
}

test_inference! {
    name = unknown_function_is_mixed,
    cases = { "<?php undefined_function();" => "mixed" }
}

test_inference! {
    name = string_naming_a_function_is_callable,
    def = "<?php function takes_int(int $x): string { return ''; }",
    cases = {
        "<?php 'takes_int'(1);" => "string",
        "<?php $f = 'takes_int'; $f(1);" => "string",
    }
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

test_inference! {
    name = instantiates_a_generic_methods_return_from_its_argument,
    def = indoc! {"
        <?php
        class Box {
            /**
             * @template T
             * @param T $value
             * @return T
             */
            public function wrap($value) { return $value; }
        }
    "},
    cases = {
        "<?php /** @var Box */ $b = null; $b->wrap(5);" => "int(5)",
        "<?php /** @var Box */ $b = null; $b->wrap('hi');" => "string('hi')",
    }
}

test_inference! {
    name = instantiates_a_generic_static_methods_return_from_its_argument,
    def = indoc! {"
        <?php
        class Factory {
            /**
             * @template T
             * @param T $value
             * @return T
             */
            public static function of($value) { return $value; }
        }
    "},
    cases = { "<?php Factory::of(true);" => "true" }
}

test_inference! {
    name = first_class_callables_are_closures_with_the_callee_signature,
    def = indoc! {"
        <?php
        function greet(string $name): string { return $name; }

        class C {
            public function size(int $n): int { return $n; }
            public static function make(): C { return new C(); }
        }
    "},
    cases = {
        "<?php greet(...);" => "(closure(string): string)",
        "<?php /** @var C */ $c = null; $c->size(...);" => "(closure(int): int)",
        "<?php C::make(...);" => "(closure(): C)",
    }
}

test_inference! {
    name = a_first_class_callable_from_an_unknown_callee_is_mixed,
    cases = { "<?php unknown(...);" => "mixed" }
}
