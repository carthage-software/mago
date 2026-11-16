<?php

declare(strict_types=1);

function list_offsets(): void
{
    $list = [1, 2];
    while ($list !== []) {
        if (rand(0, 1) === 1) {
            unset($list[0], $list[1]);
        }
    }
}

function shape_key(): void
{
    $shape = ['a' => 1];
    while ($shape !== []) {
        if (rand(0, 1) === 1) {
            unset($shape['a']);
        }
    }
}

function nested_key(): void
{
    $nested = ['inner' => ['a' => 1]];
    while ($nested['inner'] !== []) {
        if (rand(0, 1) === 1) {
            unset($nested['inner']['a']);
        }
    }
}

/**
 * @param array{1: int}|list{int, int} $values
 */
function unset_key_some_shapes_lack(array $values): int
{
    unset($values[0]);

    return count($values);
}
