<?php

declare(strict_types=1);

use BcMath\Number;

function gmp_with_gmp(GMP $a, GMP $b): GMP
{
    return ($a + $b) - ($a * $b) + ($a / $b) + ($a % $b) + ($a ** $b);
}

function gmp_with_int(GMP $a, int $b): GMP
{
    return $a + $b;
}

function gmp_with_bool(GMP $a, bool $b): GMP
{
    return $a * $b;
}

function gmp_with_integer_strings(GMP $a): GMP
{
    return $a + '0x10' + '-5' + ' 5 ' + '0b101' + '017';
}

/**
 * @param numeric-string $b
 */
function gmp_with_numeric_string(GMP $a, string $b): GMP
{
    return $a - $b;
}

function int_with_gmp(int $a, GMP $b): GMP
{
    return ($a - $b) + ($a * $b) + (2 ** $b) + ('0x10' - $b);
}

function gmp_bitwise(GMP $a, GMP $b): GMP
{
    return ($a & $b) | ($a ^ 3) | (1 & $b);
}

function gmp_shifts(GMP $a): GMP
{
    return ($a << 2) >> 1;
}

/**
 * @param non-negative-int $b
 */
function gmp_power_and_shift_by_non_negative_int(GMP $a, int $b): GMP
{
    return ($a ** $b) << $b;
}

function int_shifted_by_gmp(GMP $a): GMP
{
    return 1 << $a;
}

function gmp_or_int_with_int(GMP|int $a): GMP|int
{
    return $a * 2;
}

function gmp_compound_assignment(GMP $a): GMP
{
    $a += 1;
    $a <<= 2;
    $a |= '0x10';
    $a **= 2;

    return $a;
}

function gmp_with_float(GMP $a, float $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + $b;
}

function float_with_gmp(float $a, GMP $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a - $b;
}

function gmp_or_float_with_gmp(GMP|float $a, GMP $b): mixed
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a * $b;
}

function gmp_with_fractional_string(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + '1.5';
}

function gmp_with_non_numeric_string(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + 'abc';
}

function gmp_with_plus_signed_string(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + '+5';
}

function gmp_with_number(GMP $a, Number $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + $b;
}

function number_with_gmp(Number $a, GMP $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a * $b;
}

function gmp_bitwise_with_number(GMP $a, Number $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a & $b;
}

function gmp_with_object(GMP $a, stdClass $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + $b;
}

function gmp_divided_by_zero(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a / 0;
}

function gmp_modulo_zero_string(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a % '0';
}

function gmp_to_negative_power(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a ** -1;
}

function gmp_shifted_by_negative(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a << -1;
}

function gmp_to_hexadecimal_string_power(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a ** '0x2';
}

function gmp_compound_division_by_zero(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand,mixed-assignment */
    $a /= 0;

    return $a;
}
