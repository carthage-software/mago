<?php

declare(strict_types=1);

/**
 * @throws LogicException
 * @throws DivisionByZeroError
 * @throws ArithmeticError
 */
function finally_narrowing_reaches_code_after(?string $s, int $a, int $b): string
{
    try {
        echo intdiv($a, $b);
    } finally {
        if ($s === null) {
            throw new LogicException('missing');
        }
    }

    return $s;
}

/**
 * @throws DivisionByZeroError
 * @throws ArithmeticError
 */
function finally_unset_reaches_code_after(int $a, int $b): int
{
    $c = 1;
    try {
        echo intdiv($a, $b);
    } finally {
        unset($c);
    }

    // @mago-expect analysis:possibly-undefined-variable
    // @mago-expect analysis:mixed-return-statement
    return $c;
}
