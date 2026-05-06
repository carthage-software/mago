<?php

function test(string $type, int $zero = 0): int
{
    return $zero;
}

array_map(test(...), ['a', 'b']);
