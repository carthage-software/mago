<?php

declare(strict_types=1);

function modulo_by_half(int $a): void
{
    /**
     * @mago-expect analysis:invalid-operand
     * @mago-expect analysis:no-value
     * @mago-expect analysis:implicit-float-to-int-cast
     */
    echo $a % 0.5;
}

function modulo_by_negative_half(int $a): void
{
    /**
     * @mago-expect analysis:invalid-operand
     * @mago-expect analysis:no-value
     * @mago-expect analysis:implicit-float-to-int-cast
     */
    echo $a % -0.9;
}

function shift_by_negative_float(int $a): void
{
    /**
     * @mago-expect analysis:invalid-operand
     * @mago-expect analysis:no-value
     */
    echo $a << -2.0;
}

function valid(int $a): int
{
    return ($a % 3.0) + ($a >> 2.0);
}
