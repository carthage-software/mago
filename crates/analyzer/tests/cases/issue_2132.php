<?php

declare(strict_types=1);

/**
 * @param array<array-key, mixed> $a
 *
 * @return array{a?: int, ...<array-key, mixed>}
 */
function test(array $a): array
{
    if (isset($a['a']) && !\is_int($a['a'])) {
        exit();
    }

    return $a;
}

/**
 * @param array<array-key, mixed> $a
 *
 * @return array{name?: string, ...<array-key, mixed>}
 */
function test_with_string(array $a): array
{
    if (isset($a['name']) && !\is_string($a['name'])) {
        exit();
    }

    return $a;
}

/**
 * @param array<array-key, mixed> $a
 *
 * @return array{name?: non-empty-string, ...<array-key, mixed>}
 */
function test_with_compound_check(array $a): array
{
    if (isset($a['name']) && (!\is_string($a['name']) || $a['name'] === '')) {
        exit();
    }

    return $a;
}

/**
 * @return non-empty-string
 */
function test_exclusion_before_type(mixed $s): string
{
    if ($s === '' || !\is_string($s)) {
        exit();
    }

    return $s;
}
