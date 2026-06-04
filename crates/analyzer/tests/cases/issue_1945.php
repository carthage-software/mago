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

/**
 * @param array<int>            $in1
 * @param non-empty-list<mixed> $in2
 *
 * @return non-empty-list<int>
 *
 * @throws Exception
 */
function test1945_iterable(array $in1, array $in2): array
{
    return intersect1945($in1, $in2);
}

/**
 * @param array<int>            $in1
 * @param non-empty-list<mixed> $in2
 *
 * @return list<string>
 *
 * @throws Exception
 *
 * @mago-expect analysis:invalid-return-statement
 */
function test1945_iterable_precise(array $in1, array $in2): array
{
    return intersect1945($in1, $in2);
}

/**
 * @param list<int>    $in1
 * @param list<string> $in2
 *
 * @return list<never>
 *
 * @throws Exception
 */
function test1945_iterable_empty(array $in1, array $in2): array
{
    return intersect1945($in1, $in2);
}
