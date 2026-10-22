<?php

declare(strict_types=1);

if (count($argv) < 2 || count($argv) > 3) {
    exit(1);
}

/** @param non-empty-list<string> $arguments */
function checkLessThan(array $arguments): bool
{
    return count($arguments) < 2 || count($arguments) > 3;
}

/** @param non-empty-list<string> $arguments */
function checkLessThanOrEqual(array $arguments): bool
{
    return count($arguments) <= 1 || count($arguments) > 3;
}

/** @param non-empty-list<string> $arguments */
function checkReversedGreaterThan(array $arguments): bool
{
    return 2 > count($arguments) || 3 < count($arguments);
}

/** @param non-empty-list<string> $arguments */
function checkReversedGreaterThanOrEqual(array $arguments): bool
{
    return 1 >= count($arguments) || 3 < count($arguments);
}

/** @param non-empty-list<string> $arguments */
function checkNegatedUpperBound(array $arguments): bool
{
    return !(count($arguments) < 2) && count($arguments) < 4;
}

/**
 * @param list<string> $arguments
 * @return non-empty-list<string>
 */
function requireArguments(array $arguments): array
{
    if (count($arguments) <= 0) {
        exit(1);
    }

    return $arguments;
}

/**
 * @param list<string> $arguments
 * @return non-empty-list<string>
 */
function requireArgumentsReversed(array $arguments): array
{
    if (1 > count($arguments)) {
        exit(1);
    }

    return $arguments;
}
