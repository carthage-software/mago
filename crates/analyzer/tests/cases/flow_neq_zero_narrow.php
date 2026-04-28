<?php

declare(strict_types=1);

/**
 * @param 0|1|2|3 $value
 *
 * @throws \DivisionByZeroError
 * @throws \ArithmeticError
 */
function flow_neq_zero_narrow(int $value): int
{
    if ($value !== 0) {
        return intdiv(100, $value);
    }

    return 0;
}
