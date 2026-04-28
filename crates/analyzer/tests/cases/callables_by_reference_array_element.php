<?php

declare(strict_types=1);

function callables_double_ref(int &$n): void
{
    $n *= 2;
}

$arr = ['x' => 5];
callables_double_ref($arr['x']);
echo $arr['x'];
