<?php

declare(strict_types=1);

/**
 * @return list<int>
 */
function offset_write_append_creates_list(): array
{
    $values[] = 1;
    $values[] = 2;

    return $values;
}

/**
 * @return array{k: int, n: array{m: string}}
 */
function offset_write_key_creates_array(): array
{
    $values['k'] = 1;
    $values['n']['m'] = 'a';

    return $values;
}
