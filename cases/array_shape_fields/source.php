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
 *  self: int,
 *  static: int,
 *  parent: int,
 *  true: int,
 *  false: int,
 *  null: int,
 *  0b1010: int,
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
        'self' => 1,
        'static' => 2,
        'parent' => 3,
        'true' => 1,
        'false' => 0,
        'null' => 0,
        0b1010 => 10,
    ]; // no `float` key as it is optional
}
