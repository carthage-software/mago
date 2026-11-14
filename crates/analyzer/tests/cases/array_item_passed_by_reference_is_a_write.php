<?php

declare(strict_types=1);

/**
 * @return list<array<array-key, string>>
 */
function append_by_reference(string $s): array
{
    $a = [];
    preg_match('/x/', $s, $a[]);

    return $a;
}

function new_key_by_reference(string $s): string
{
    $o = ['x' => 'y'];
    preg_match('/a/', $s, $o['m']);

    return $o['x'];
}

/**
 * @return array{m: array<array-key, string>}
 */
function first_key_by_reference(string $s): array
{
    $o = [];
    preg_match('/a/', $s, $o['m']);

    return $o;
}

/**
 * @param array<string, array<array-key, string>>|null $a
 */
function nullable_container(?array $a, string $s): void
{
    preg_match('/a/', $s, $a['m']);
}
