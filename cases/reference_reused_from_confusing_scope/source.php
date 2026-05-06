<?php

function foo(): int
{
    $array = [1, 2, 3];
    foreach ($array as &$value) {
        $value = $value + 1;
    }

    $value = 2;

    return $value;
}

function bar(): int
{
    $array = [1, 2, 3];
    foreach ($array as &$value) {
        $value = $value + 1;
    }
    unset($value); // Break the reference with the last element

    $value = 2;

    return $value;
}
