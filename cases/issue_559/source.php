<?php

/**
 * @param array{0: string, a: string} $arr
 */
function test(array $arr): void
{
    echo $arr['a'];
}

$array = ['foo'];

$array['a'] = 'b';

test($array);
