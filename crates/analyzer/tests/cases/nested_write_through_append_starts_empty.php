<?php

declare(strict_types=1);

/**
 * @return list{int, array{k: int}}
 */
function on_list(): array
{
    $list = [1];
    $list[]['k'] = 2;

    return $list;
}

/**
 * @return array{A: string, 0: array{C: array{}}}
 */
function on_shape(): array
{
    $shape = ['A' => 'B'];
    $shape[]['C'] = [];

    return $shape;
}

/**
 * @param list<int> $list
 *
 * @return non-empty-list<int|array{k: int}>
 */
function on_list_parameter(array $list): array
{
    $list[]['k'] = 2;

    return $list;
}
