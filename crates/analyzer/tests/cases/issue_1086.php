<?php

declare(strict_types=1);

/**
 * @param non-empty-uppercase-string $input
 */
function test(string $input): bool
{
    return $input === '123';
}

/**
 * @param non-empty-uppercase-string $input
 */
function test2(string $input): bool
{
    return $input === 'A 123';
}

/**
 * @param non-empty-uppercase-string $input
 */
function test3(string $input): bool
{
    return $input === 'A ';
}

/**
 * @param non-empty-uppercase-string $input
 */
function test4(string $input): bool
{
    return $input === 'A';
}

/**
 * @param non-empty-lowercase-string $input
 */
function test5(string $input): bool
{
    return $input === '123';
}

/**
 * @param non-empty-lowercase-string $input
 */
function test6(string $input): bool
{
    return $input === 'a 123';
}

/**
 * @param non-empty-lowercase-string $input
 */
function test7(string $input): bool
{
    return $input === 'a ';
}

/**
 * @param non-empty-lowercase-string $input
 */
function test8(string $input): bool
{
    return $input === 'a';
}
