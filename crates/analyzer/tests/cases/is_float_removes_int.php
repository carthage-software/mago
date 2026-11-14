<?php

declare(strict_types=1);

function is_float_takes_float(float $value): float
{
    return $value;
}

function is_float_of_int_or_float(int|float $value): float
{
    if (is_float($value)) {
        // @mago-expect analysis:impossible-condition
        if (is_int($value)) {
            return 0.0;
        }

        return is_float_takes_float($value);
    }

    return 0.0;
}

/**
 * @param list<int|float> $values
 */
function is_float_filter(array $values): float
{
    $floats = array_filter($values, is_float(...));
    $first = $floats[0] ?? 0.0;

    // @mago-expect analysis:impossible-condition
    if (is_int($first)) {
        return 1.0;
    }

    return $first;
}
