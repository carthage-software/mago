<?php

declare(strict_types=1);

use BcMath\Number;

function number_sum(Number $a, Number $b): Number
{
    /** @mago-expect analysis:invalid-operand */
    $sum = $a + $b;

    return $sum;
}

function gmp_times_int(GMP $a): GMP
{
    /** @mago-expect analysis:invalid-operand */
    $product = $a * 2;

    return $product;
}

function int_minus_gmp(GMP $a): GMP
{
    /** @mago-expect analysis:invalid-operand */
    $difference = 10 - $a;

    return $difference;
}

function gmp_bitwise_and_shift(GMP $a, GMP $b): GMP
{
    /** @mago-expect analysis:invalid-operand(2) */
    $result = ($a & $b) << 2;

    return $result;
}

function gmp_compound_addition(GMP $a): GMP
{
    /** @mago-expect analysis:invalid-operand */
    $a += 1;

    return $a;
}

function gmp_or_int_times_int(GMP|int $a): GMP|int
{
    /** @mago-expect analysis:possibly-invalid-operand */
    $product = $a * 2;

    return $product;
}

function number_negated(Number $a): Number
{
    /** @mago-expect analysis:invalid-operand */
    $negated = -$a;

    return $negated;
}

function gmp_plus(GMP $a): GMP
{
    /** @mago-expect analysis:invalid-operand */
    $result = +$a;

    return $result;
}

function gmp_bitwise_not(GMP $a): GMP
{
    /** @mago-expect analysis:invalid-operand */
    $result = ~$a;

    return $result;
}

function number_or_int_negated(Number|int $a): Number|int
{
    /** @mago-expect analysis:possibly-invalid-operand */
    $negated = -$a;

    return $negated;
}

function gmp_incremented(GMP $a): GMP
{
    /** @mago-expect analysis:invalid-operand */
    $a++;

    return $a;
}

function number_decremented(Number $a): Number
{
    /** @mago-expect analysis:invalid-operand */
    --$a;

    return $a;
}

/**
 * @return list<bool|int>
 */
function comparisons_are_not_reported(Number $a, Number $b, GMP $c): array
{
    return [$a < 5, $a >= $b, $a == '1.5', $c > 3, $c != 3, $a <=> $b, $c <=> 3];
}

function gmp_plus_float_is_reported_once(GMP $a): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    $sum = $a + 1.5;

    return $sum;
}

function gmp_or_float_times_gmp_is_reported_once(GMP|float $a, GMP $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    $product = $a * $b;

    return $product;
}

function gmp_with_non_numeric_string(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + 'abc';
}

function gmp_divided_by_zero(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand,never-return */
    return $a / 0;
}

function number_bitwise_not(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand,never-return */
    return ~$a;
}

function gmp_or_number_bitwise_not(GMP|Number $a): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return ~$a;
}

function gmp_less_than_array(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a < [];
}

function gmp_equal_to_non_integer_string(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a == 'abc';
}
