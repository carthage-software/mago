<?php

/** @return array{a: string, b: string} */
function foo(): array
{
    $a = 'hello';
    $b = 'world';

    return compact('a', 'b');
}

$result = foo();
echo $result['a'];
echo $result['b'];
