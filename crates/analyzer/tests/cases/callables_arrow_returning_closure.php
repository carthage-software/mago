<?php

declare(strict_types=1);

/**
 * @return Closure(int): int
 */
function callables_curry_add(int $a): Closure
{
    return fn(int $b): int => $a + $b;
}

$add3 = callables_curry_add(3);
echo $add3(4);
