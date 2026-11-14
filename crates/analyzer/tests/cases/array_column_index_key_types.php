<?php

declare(strict_types=1);

/** @param list<array{id: int|string, name: string}> $rows */
function by_id(array $rows): string
{
    $names = array_column($rows, 'name', 'id');

    return $names['abc'] ?? 'none';
}

/** @param list<array{id: int|string, name: string}> $rows */
function keys(array $rows): void
{
    foreach (array_column($rows, 'name', 'id') as $id => $name) {
        if (is_string($id)) {
            echo $id, $name;
        }
    }
}

/**
 * @param list<array{id: int|string, name: string}> $rows
 *
 * @return list<string>
 */
function as_list(array $rows): array
{
    // @mago-expect analysis:invalid-return-statement
    return array_column($rows, 'name', 'id');
}

function literal_rows(): void
{
    $names = array_column([['id' => 10, 'name' => 'a'], ['id' => 20, 'name' => 'b']], 'name', 'id');

    echo count($names);
}
