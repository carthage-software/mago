<?php

declare(strict_types=1);

/** @param positive-int $p */
function power_of_positive_exponent(int $base, int $p): int
{
    return pow($base, $p) + pow($base, 2);
}

function power_of_negative_exponent(int $base): float
{
    return pow($base, -1);
}

/** @return 1024 */
function literal_power(): int
{
    return pow(2, 10);
}

function float_power(float $base): float
{
    return pow($base, 2);
}

/** @param 'a'|'b' $k */
function check_lower(string $k): void {}

/** @param 'A'|'B' $k */
function check_upper(string $k): void {}

/** @param 'A'|'B' $k */
function lower(string $k): void
{
    check_lower(strtolower($k));
    check_lower(mb_strtolower($k));
    check_upper(strtoupper(strtolower($k)));
    check_upper(mb_strtoupper(strtolower($k)));
}
