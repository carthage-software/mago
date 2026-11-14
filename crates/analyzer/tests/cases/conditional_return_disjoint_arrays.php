<?php

declare(strict_types=1);

/**
 * @param array{kind: 'a'|'b'} $v
 * @return ($v is array{kind: 'a'} ? string : int)
 */
function conditional_disjoint_by_shape(array $v): string|int
{
    return $v['kind'] === 'a' ? 'a' : 1;
}

/**
 * @param list<int>|list<string> $v
 * @return ($v is list<int> ? string : int)
 */
function conditional_disjoint_by_list(array $v): string|int
{
    return 1;
}

function conditional_disjoint_shape_b(): int
{
    return conditional_disjoint_by_shape(['kind' => 'b']);
}

function conditional_disjoint_list_of_strings(): int
{
    return conditional_disjoint_by_list(['a', 'b']);
}
