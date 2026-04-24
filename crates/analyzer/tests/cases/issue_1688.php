<?php

declare(strict_types=1);

/**
 * @param non-empty-string $a
 * @param non-empty-string $b
 * @return non-empty-string
 */
function return_string(string $a, string $b): string
{
    return sprintf('%s%s', $a, $b);
}

/**
 * @param non-empty-string $a
 * @return non-empty-string
 */
function single_non_empty(string $a): string
{
    return sprintf('%s', $a);
}

/**
 * @return non-empty-string
 */
function integer_specifier(int $n): string
{
    return sprintf('%d', $n);
}
