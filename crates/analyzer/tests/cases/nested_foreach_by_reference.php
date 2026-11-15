<?php

declare(strict_types=1);

/**
 * @param array<string, array<string, array<string, string>>> $data
 * @return array<string, array<string, array<string, string>>>
 */
function nested(array $data): array
{
    foreach ($data as &$group) {
        foreach ($group as &$item) {
            $item = ['href' => 'x'];
        }
    }

    return $data;
}

/**
 * @param list<int> $first
 * @param list<int> $second
 * @return list<int>
 */
function sequential(array $first, array $second): array
{
    foreach ($first as &$value) {
        $value++;
    }

    foreach ($second as &$value) {
        $value--;
    }

    return [...$first, ...$second];
}

/**
 * @param list<int> $values
 * @return list<int>
 */
function write_after_loop(array $values): array
{
    foreach ($values as &$value) {
        $value++;
    }

    // @mago-expect analysis:reference-reused-from-confusing-scope
    $value = 0;

    return $values;
}
