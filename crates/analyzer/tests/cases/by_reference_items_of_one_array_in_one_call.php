<?php

declare(strict_types=1);

/**
 * @param-out int $code
 * @param-out string $out
 */
function run(mixed &$code, mixed &$out): void
{
    $code = 0;
    $out = '';
}

/**
 * @return array{code: int, out: string}
 */
function from_nulls(): array
{
    $o = ['code' => null, 'out' => null];
    run($o['code'], $o['out']);

    return $o;
}

/**
 * @return array{code: int, out: string}
 */
function from_empty(): array
{
    $o = [];
    run($o['code'], $o['out']);

    return $o;
}

/**
 * @return array{result: array{code: int, out: string}}
 */
function nested(): array
{
    $o = ['result' => []];
    run($o['result']['code'], $o['result']['out']);

    return $o;
}
