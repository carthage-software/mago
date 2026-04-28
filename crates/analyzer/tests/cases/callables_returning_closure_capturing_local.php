<?php

declare(strict_types=1);

/**
 * @return Closure(int): int
 */
function callables_make_adder(int $offset): Closure
{
    return function (int $n) use ($offset): int {
        return $n + $offset;
    };
}

$add5 = callables_make_adder(5);
echo $add5(10);
