<?php

declare(strict_types=1);

/**
 * @return ($v is int ? int : string)
 */
function conditional_int_float_describe(int|float|null $v): int|string
{
    if (is_int($v)) {
        return $v;
    }

    return $v === null ? 'null' : 'float';
}

function conditional_int_float_f(?float $f): string
{
    return conditional_int_float_describe($f);
}
