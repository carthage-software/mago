<?php

/**
 * @return array{
 *  'literal-string-key': string,
 *  1: int,
 *  -2: int,
 *  +4: int,
 *  -1.2: float,
 *  +1.2: float,
 *  unquoted-key: string,
 *  list: list<int>,
 *  int: int,
 *  float?: float,
 * }
 */
function example(): array
{
    return [
        'literal-string-key' => 'value',
        1 => 42,
        -2 => -42,
        +4 => 84,
        '-1.2' => -1.2,
        '+1.2' => 1.2,
        'unquoted-key' => 'value',
        'list' => [1, 2, 3],
        'int' => 100,
    ]; // no `float` key as it is optional
}
