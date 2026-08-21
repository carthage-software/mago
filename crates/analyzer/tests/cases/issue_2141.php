<?php

declare(strict_types=1);

namespace Psl\Type {
    /** @template-covariant T */
    interface TypeInterface
    {
        /**
         * @param mixed $value
         * @return T
         */
        public function assert($value): mixed;
    }

    /** @return TypeInterface<string> */
    function string(): TypeInterface
    {
        return string();
    }

    /** @return TypeInterface<non-empty-string> */
    function non_empty_string(): TypeInterface
    {
        return non_empty_string();
    }
}

namespace {
    use Psl\Type;

    function issue_2141_get_mixed(): mixed
    {
        return 'mixed';
    }

    function issue_2141_take_string(string $_): void {}

    $foo = Type\string()->assert(issue_2141_get_mixed());
    $foo = trim($foo);
    $foo = Type\non_empty_string()->assert($foo);

    $bar = Type\string()->assert(issue_2141_get_mixed()) |> trim(...) |> Type\non_empty_string()->assert(...);
    $assert_non_empty = Type\non_empty_string()->assert(...);
    $baz = $assert_non_empty($foo);

    issue_2141_take_string($foo);
    issue_2141_take_string($bar);
    issue_2141_take_string($baz);
}
