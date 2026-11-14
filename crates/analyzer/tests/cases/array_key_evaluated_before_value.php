<?php

declare(strict_types=1);

/**
 * @return array<int, object>
 */
function array_key_before_value_roots(object $node): array
{
    $roots = [];
    $roots[spl_object_id($root = $node)] = $root;

    return $roots;
}

/**
 * @return array{0: 1}
 */
function array_key_before_value_increment(): array
{
    $i = 0;
    $a = [];
    $a[$i++] = $i;

    return $a;
}

final class ArrayKeyBeforeValue
{
    /** @var array<string, string> */
    public array $p = [];
}

function array_key_before_value_property(ArrayKeyBeforeValue $o): void
{
    $o->p[$k = 'x'] = $k;
}
