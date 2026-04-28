<?php

declare(strict_types=1);

function flow_is_float_narrow(int|float $value): float
{
    if (is_float($value)) {
        return $value;
    }

    return (float) $value;
}
