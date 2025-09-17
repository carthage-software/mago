<?php

function foo(string $a, int $b): void
{
    var_dump($a, $b);
}

foo(...['a' => 'a', 'b' => 2]);
