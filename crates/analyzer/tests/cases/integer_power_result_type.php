<?php

declare(strict_types=1);

function literal_power(): int
{
    return 2 ** 10;
}

function negative_literal_exponent(): float
{
    return 2 ** (-1);
}

function overflowing_literal_power(): float
{
    return 2 ** 64;
}

/** @param int<0, max> $exponent */
function non_negative_exponent(int $base, int $exponent): int
{
    return $base ** $exponent;
}

/** @param negative-int $exponent */
function negative_exponent(int $base, int $exponent): float
{
    return $base ** $exponent;
}

function unknown_exponent(int $base, int $exponent): int|float
{
    return $base ** $exponent;
}

function wrong_half(): int
{
    // @mago-expect analysis:invalid-return-statement
    return 2 ** (-1);
}

function wrong_unknown(int $base, int $exponent): int
{
    // @mago-expect analysis:invalid-return-statement
    return $base ** $exponent;
}
