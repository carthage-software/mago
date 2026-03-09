<?php

declare(strict_types=1);

/** @return array{mykey1: 'myval1'}|array{mykey2: 'myval2'} */
function test_disjoint(): array
{
    return rand(min: 0, max: 1) ? ['mykey1' => 'myval1'] : ['mykey2' => 'myval2'];
}

/** @return array{a: int}|array{b: string}|array{c: float} */
function test_three_branches(): array
{
    $r = rand(0, 2);
    if ($r === 0) {
        return ['a' => 1];
    } elseif ($r === 1) {
        return ['b' => 'hello'];
    }

    return ['c' => 1.5];
}

/** @return array{status: string, data?: int} */
function test_overlapping(): array
{
    if (rand(0, 1)) {
        return ['status' => 'ok', 'data' => 42];
    }

    return ['status' => 'error'];
}

/** @return array{name: string}|array{id: int} */
function test_variable(): array
{
    $result = rand(0, 1) ? ['name' => 'Alice'] : ['id' => 123];

    return $result;
}

/** @return array{a: int, b: int}|array{c: string, d: string} */
function test_multi_key_disjoint(): array
{
    return rand(0, 1) ? ['a' => 1, 'b' => 2] : ['c' => 'x', 'd' => 'y'];
}
