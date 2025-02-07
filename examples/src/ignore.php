<?php

declare(strict_types=1);

namespace Mago\Examples;

/**
 * @mago-ignore analysis/undefined-function-or-method
 * @mago-ignore strictness/require-return-type
 */
function example()
{
    $a = foo();
    // FIXME: This is a test
    $b = bar();
    $c = baz();
}

/**
 * @mago-ignore strictness/require-parameter-type
 * @mago-ignore best-practices/no-unused-parameter
 * @mago-ignore comment/no-untagged-todo
 */
function example2($foo): void
{
    // TODO: This is a test
    // @mago-ignore redundancy/redundant-block
    {
        $a = foo();
    }
    $b = bar();
    $c =
        baz(); // @mago-ignore analysis/undefined-function-or-method

    // @mago-ignore analysis/undefined-function-or-method
    $a = [
        'foo' => Foo::Bar, // @mago-ignore analysis/undefined-constant-or-enum-case
        'bar' => bar(), // @mago-ignore analysis/undefined-function-or-method
        'baz' => baz(), // @mago-ignore analysis/undefined-function-or-method
    ];
}
