<?php

declare(strict_types=1);

function f(string $s): void
{
    $first = 'abc';
    $second = 'abd';
    if ($first == $second) {
        echo 1;
    }

    $one = '1';
    $padded = '01';
    if ($one != $padded) {
        echo 2;
    }

    if ($s == $first) {
        echo 3;
    }
}
