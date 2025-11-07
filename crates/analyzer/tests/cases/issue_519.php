<?php

/**
 * @param array{foo: string} $d
 */
function take_one(array $d): void
{
    echo $d['foo'];
}

/**
 * @param array{baz: string} $d
 */
function take_two(array $d): void
{
    echo $d['baz'];
}

/**
 * @param array{foo: string, baz: string} $d
 */
function take_both(array $d): void
{
    echo $d['foo'] . ' ' . $d['baz'];
}

$dict1 = ['foo' => 'bar'];
$dict2 = ['baz' => 'qux'];
$dict3 = array_merge($dict1, $dict2);

take_one($dict1);
take_two($dict2);
take_both($dict3);
