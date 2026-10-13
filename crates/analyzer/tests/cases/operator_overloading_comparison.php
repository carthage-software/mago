<?php

declare(strict_types=1);

use BcMath\Number;

/**
 * @return list<bool|int>
 */
function gmp_compared_with_accepted_operands(GMP $a, GMP $b, int $i, bool $flag): array
{
    return [
        $a < $i,
        $a <= '5',
        $a > ' 5',
        $a >= '0x5',
        $a == $b,
        $a != $flag,
        $a <> 5,
        $i < $a,
        '5' == $a,
        $a <=> $i,
        $a <=> $b,
        '-5' <=> $a,
    ];
}

/**
 * @return list<bool|int>
 */
function number_compared_with_accepted_operands(Number $a, Number $b, int $i, string $s, array $array): array
{
    return [
        $a < $i,
        $a <= '5.5',
        $a == $b,
        $a != $s,
        $a == $array,
        $a == new stdClass(),
        $a <=> $i,
        $a <=> $b,
    ];
}

/**
 * @return list<bool|int>
 */
function comparisons_whose_handler_accepts_the_other_operand(GMP $g, Number $n, stdClass $o): array
{
    return [
        $n == $g,
        $g > $n,
        $g >= $n,
        $n < 'abc',
        $n <=> $g,
        5 < $g,
        $o < $g,
    ];
}

function gmp_identical_to_non_integer_string(GMP $a): bool
{
    /** @mago-expect analysis:redundant-comparison */
    return $a === 'abc';
}

function gmp_not_identical_to_array(GMP $a): bool
{
    /** @mago-expect analysis:redundant-comparison */
    return $a !== [];
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

function gmp_not_equal_to_decimal_string(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a != '1.5';
}

function gmp_less_than_float(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a < 1.5;
}

function gmp_equal_to_number(GMP $a, Number $b): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a == $b;
}

function gmp_not_equal_to_object(GMP $a, stdClass $b): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a <> $b;
}

function number_greater_than_gmp(Number $a, GMP $b): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a > $b;
}

function array_less_than_gmp(GMP $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return [] < $a;
}

function non_integer_string_spaceship_gmp(GMP $a): int
{
    /** @mago-expect analysis:invalid-operand */
    return 'abc' <=> $a;
}

function gmp_spaceship_array(GMP $a): int
{
    /** @mago-expect analysis:invalid-operand */
    return $a <=> [];
}

function number_less_than_float(Number $a): bool
{
    /** @mago-expect analysis:invalid-operand */
    return $a < 1.5;
}

function gmp_or_int_less_than_array(GMP|int $a): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a < [];
}

function gmp_compared_with_int_or_array(GMP $a, int|array $b): bool
{
    /** @mago-expect analysis:possibly-invalid-operand */
    return $a == $b;
}
