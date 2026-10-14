<?php

declare(strict_types=1);

use BcMath\Number;

function gmp_negated(GMP $a): GMP
{
    return -$a;
}

function gmp_plus(GMP $a): GMP
{
    return +$a;
}

function gmp_bitwise_not(GMP $a): GMP
{
    return ~$a;
}

function number_negated(Number $a): Number
{
    return -$a;
}

function number_plus(Number $a): Number
{
    return +$a;
}

/**
 * @return GMP|0
 */
function nullable_gmp_negated(?GMP $a): GMP|int
{
    return -$a;
}

function gmp_or_int_negated(GMP|int $a): GMP|int
{
    return -$a;
}

function gmp_incremented_and_decremented(GMP $a): GMP
{
    $a++;
    $a--;
    ++$a;

    return --$a;
}

function number_incremented_and_decremented(Number $a): Number
{
    $a++;
    $a--;
    --$a;

    return ++$a;
}

function gmp_post_increment_value(GMP $a): GMP
{
    return $a++;
}

/**
 * @return GMP|1
 */
function nullable_gmp_incremented(?GMP $a): GMP|int
{
    $a++;

    return $a;
}

function gmp_incremented_in_loop(GMP $a, int $times): GMP
{
    for ($i = 0; $i < $times; $i++) {
        $a++;
    }

    return $a;
}

function number_decremented_in_loop(Number $a, int $times): Number
{
    while ($times-- > 0) {
        $a--;
    }

    return $a;
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

function object_negated(stdClass $a): mixed
{
    /** @mago-expect analysis:invalid-operand,never-return */
    return -$a;
}

function object_incremented(stdClass $a): mixed
{
    /** @mago-expect analysis:invalid-operand,mixed-assignment */
    $a++;

    return $a;
}

function object_decremented(stdClass $a): mixed
{
    /** @mago-expect analysis:invalid-operand,mixed-assignment */
    $a--;

    return $a;
}
