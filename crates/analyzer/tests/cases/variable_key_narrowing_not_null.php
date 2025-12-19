<?php

/**
 * @param array<string, int|null> $map
 */
function testNotNullVariableKeyNarrowing(array $map, string $key): int
{
    if ($map[$key] !== null) {
        return $map[$key];
    }

    return 0;
}

/**
 * @param array<string, array<string, int|null>> $map
 */
function testNotNullNestedVariableKeyNarrowing(array $map, string $key1, string $key2): int
{
    if ($map[$key1][$key2] !== null) {
        return $map[$key1][$key2];
    }

    return 0;
}
