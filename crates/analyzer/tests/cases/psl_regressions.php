<?php

declare(strict_types=1);

function remove_dot_segments(string $path): string
{
    $output = [];
    $segments = explode('/', $path);

    foreach ($segments as $segment) {
        if ($segment === '.') {
            continue;
        }

        if ($segment === '..') {
            if ($output !== []) {
                array_pop($output);
            }

            continue;
        }

        $output[] = $segment;
    }

    return implode('/', $output);
}

/**
 * @param list<non-empty-string> $current
 * @param non-empty-list<lowercase-string> $expected
 */
function needs_update(array $current, array $expected): bool
{
    return $current !== $expected;
}

/**
 * @param list<non-empty-string> $current
 * @param non-empty-list<lowercase-string> $expected
 */
function needs_update2(array $current, array $expected): bool
{
    return $current === $expected;
}

/**
 * @param non-empty-string $a
 * @param lowercase-string $b
 */
function strings_can_match(string $a, string $b): bool
{
    return $a !== $b;
}

/**
 * @param non-empty-string $a
 * @param lowercase-string $b
 */
function strings_can_match2(string $a, string $b): bool
{
    return $a === $b;
}
