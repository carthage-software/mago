<?php

function booleans_arr(bool $a, bool $b): string
{
    return match ([$a, $b]) {
        [true, true] => 'a and b',
        [true, false] => 'a and not b',
        [false, true] => 'not a and b',
        [false, false] => 'not a and not b',
    };
}

function booleans3_arr(bool $a, bool $b, bool $c): string
{
    return match ([$a, $b, $c]) {
        [true, true, true] => 'a and b and c',
        [true, true, false] => 'a and b and not c',
        [true, false, true] => 'a and not b and c',
        [true, false, false] => 'a and not b and not c',
        [false, true, true] => 'not a and b and c',
        [false, true, false] => 'not a and b and not c',
        [false, false, true] => 'not a and not b and c',
        [false, false, false] => 'not a and not b and not c',
    };
}
