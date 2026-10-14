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

function takes_int(int $value): void
{
}

function takes_gmp(GMP $value): void
{
}

function gmp_or_int_result_keeps_both_types(GMP|int $a): void
{
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_int($a * 2);
    /** @mago-expect analysis:possibly-invalid-argument */
    takes_gmp($a * 2);
}

/**
 * @template T of GMP
 *
 * @param T $a
 */
function gmp_template_with_int(GMP $a): GMP
{
    return $a + 1;
}

function nullable_gmp_with_int(?GMP $a): GMP
{
    /** @mago-expect analysis:possibly-null-operand */
    return $a + 1;
}

function gmp_compound_assignment(GMP $a): GMP
{
    $a += 1;
    $a <<= 2;
    $a |= '0x10';
    $a **= 2;

    return $a;
}

function gmp_with_float(GMP $a, float $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a + $b;
}

function float_with_gmp(float $a, GMP $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a - $b;
}

function gmp_or_float_with_gmp(GMP|float $a, GMP $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a * $b;
}

function gmp_with_bool(GMP $a, bool $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a * $b;
}

function bool_with_gmp(bool $a, GMP $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a - $b;
}

function true_to_the_power_of_gmp(GMP $a): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return true ** $a;
}

function gmp_to_the_power_of_float(GMP $a, float $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a ** $b;
}

function gmp_shifted_by_float(GMP $a): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a >> 1.5;
}

function gmp_divided_by_false(GMP $a): mixed
{
    /** @mago-expect analysis:false-operand,invalid-operand,never-return */
    return $a / false;
}

function gmp_to_the_power_of_true(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a ** true;
}

function gmp_shifted_by_false(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a << false;
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

function number_bitwise_with_gmp(Number $a, GMP $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a & $b;
}

function number_shifted_by_gmp(Number $a, GMP $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a << $b;
}

function gmp_with_object(GMP $a, stdClass $b): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + $b;
}

/**
 * @param numeric $b
 */
function gmp_with_numeric(GMP $a, mixed $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a + $b;
}

/**
 * @param numeric $a
 */
function numeric_with_gmp(mixed $a, GMP $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a * $b;
}

/**
 * @param array-key $b
 */
function gmp_with_array_key(GMP $a, int|string $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a - $b;
}

/**
 * @param scalar $b
 */
function gmp_with_scalar(GMP $a, int|float|string|bool $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a % $b;
}

function gmp_with_any_object(GMP $a, object $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a & $b;
}

/**
 * @param numeric $b
 */
function gmp_to_the_power_of_numeric(GMP $a, mixed $b): GMP
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a ** $b;
}

function gmp_divided_by_zero(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand,never-return */
    return $a / 0;
}

function gmp_modulo_zero_string(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand,never-return */
    return $a % '0';
}

function gmp_to_negative_power(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a ** -1;
}

function gmp_or_int_to_negative_power(GMP|int $a): mixed
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a ** -1;
}

function gmp_shifted_by_negative(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a << -1;
}

function gmp_shifted_by_negative_string(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a >> '-1';
}

function gmp_to_hexadecimal_string_power(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a ** '0x2';
}

function gmp_compound_division_by_zero(GMP $a): mixed
{
    /** @mago-expect analysis:invalid-operand,impossible-assignment */
    $a /= 0;

    /** @mago-expect analysis:never-return */
    return $a;
}
