<?php

declare(strict_types=1);

function divide(int $int, float $float): float
{
    /** @mago-expect analysis:invalid-operand */
    /** @mago-expect analysis:mixed-assignment */
    $a = $int / 0.0;
    /** @mago-expect analysis:invalid-operand */
    /** @mago-expect analysis:mixed-assignment */
    $b = $float / 0.0;
    $zero = 0.0;
    /** @mago-expect analysis:invalid-operand */
    /** @mago-expect analysis:mixed-assignment */
    $c = $float / $zero;

    return ($int / 2.0) + ($float / 2.0);
}
