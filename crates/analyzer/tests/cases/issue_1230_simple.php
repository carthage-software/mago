<?php

declare(strict_types=1);

/**
 * @param list<int> $items
 *
 * @return list<array{name: string, value: int}>
 */
function simpleLoop(array $items): array
{
    $result = [];

    foreach ($items as $item) {
        $result[] = [
            'name' => 'test',
            'value' => $item,
        ];
    }

    return $result;
}

/**
 * @param list<array{id: int, value: int}> $items
 *
 * @return list<array{id: int, values: list<int>}>
 */
function loopWithBranching(array $items): array
{
    $result = [];

    foreach ($items as $item) {
        $last = \array_key_last($result);

        if (null !== $last && $result[$last]['id'] === $item['id']) {
            $result[$last]['values'][] = $item['value'];
        } else {
            $result[] = [
                'id' => $item['id'],
                'values' => [$item['value']],
            ];
        }
    }

    return $result;
}
