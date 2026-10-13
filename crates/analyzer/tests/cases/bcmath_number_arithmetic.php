<?php

declare(strict_types=1);

use BcMath\Number;

function number_with_number(Number $a, Number $b): Number
{
    return $a * $b;
}

function number_with_int(Number $a, int $b): Number
{
    return $a + $b;
}

function int_with_number(int $a, Number $b): Number
{
    return ($a - $b) + ($a * $b) + ($a / $b) + ($a % $b) + ($a ** $b);
}

/**
 * @param numeric-string $b
 */
function number_with_numeric_string(Number $a, string $b): Number
{
    return $a / $b;
}

function number_with_numeric_literals(Number $a): Number
{
    return $a + '1.5' + '-2' + '1.' + '.5';
}

function number_modulo_and_power(Number $a): Number
{
    return ($a % 3) ** 2;
}

function number_to_negative_power(Number $a): Number
{
    return $a ** -1;
}

function number_with_bool(Number $a, bool $b): Number
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a + $b;
}

function number_to_the_power_of_true(Number $a): Number
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a ** true;
}

function number_divided_by_false(Number $a): mixed
{
    /** @mago-expect analysis:false-operand,invalid-operand,never-return */
    return $a / false;
}

function number_or_int_with_int(Number|int $a): Number|int
{
    return $a * 2;
}

/**
 * @template T of Number
 *
 * @param T $a
 */
function template_with_int(Number $a): Number
{
    return $a + 1;
}

function compound_assignment(Number $a): Number
{
    $a += 1;
    $a *= '2.5';

    return $a;
}

function number_with_float(Number $a, float $b): Number
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a * $b;
}

function number_or_float_with_number(Number|float $a, Number $b): Number
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a * $b;
}

/**
 * @param numeric $b
 */
function number_with_numeric(Number $a, mixed $b): Number
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a + $b;
}

function number_with_non_numeric_string(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + 'abc';
}

function number_with_exponent_literal(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + '1e3';
}

function number_with_array(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a + [];
}

function number_with_null(Number $a): mixed
{
    /** @mago-expect analysis:null-operand */
    return $a + null;
}

function number_divided_by_zero(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand,never-return */
    return $a / 0;
}

function number_modulo_zero_string(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand,never-return */
    return $a % '0';
}

function number_shifted(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a << 1;
}

function number_bitwise_and(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a & 1;
}

function number_to_fractional_string_power(Number $a): mixed
{
    /** @mago-expect analysis:invalid-operand */
    return $a ** '1.5';
}

function number_with_any_object(Number $a, object $b): Number
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a / $b;
}
