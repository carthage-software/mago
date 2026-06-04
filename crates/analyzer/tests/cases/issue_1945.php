<?php

declare(strict_types=1);

/**
 * @template A
 * @template B
 *
 * @param A $a
 * @param B $b
 *
 * @return A&B
 *
 * @throws Exception
 */
function intersect1945(mixed $a, mixed $b): mixed
{
    throw new Exception('irrelevant');
}

/**
 * @throws Exception
 */
function test1945_ok(int|bool $in1, int|float $in2): int
{
    return intersect1945($in1, $in2);
}

/**
 * @throws Exception
 *
 * @mago-expect analysis:invalid-return-statement
 */
function test1945_precise(int|bool $in1, int|float $in2): string
{
    return intersect1945($in1, $in2);
}

/**
 * @throws Exception
 *
 * @mago-expect analysis:never-return
 */
function test1945_disjoint(int $in1, string $in2): int
{
    return intersect1945($in1, $in2);
}
