<?php

/**
 * @template T of array
 * @template TKey of string
 * @param T $arr
 * @param TKey $k
 * @return T[TKey]
 */
function a(array $arr, string $k): mixed
{
    assert(isset($arr[$k]));
    /** @mago-expect analysis:mixed-return-statement */
    return $arr[$k];
}

$a = a(['test' => 123], 'test');

/** @param 123 $x */
function foo(int $x): void
{
    echo $x;
}

foo($a);
