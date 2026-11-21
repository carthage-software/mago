<?php

declare(strict_types=1);

/**
 * @param decimal-int-string $decimal
 * @param non-decimal-int-string $other
 */
function decimal_int_string_types(string $decimal, string $other): string
{
    return $decimal . $other;
}
