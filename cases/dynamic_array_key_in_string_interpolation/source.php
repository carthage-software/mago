<?php

/** @param 'bar' $a */
function f(string $a): void
{
    echo $a;
}

$a = ['foo_1' => 'bar', 'foo_2' => 'baz'];

$n = 1;
f("{$a["foo_$n"]}");
f("{$a["foo_{$n}"]}");

$n = 2;
f("{$a["foo_$n"]}");
f("{$a["foo_{$n}"]}");
