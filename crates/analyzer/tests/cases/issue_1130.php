<?php declare(strict_types=1);

function test(): string
{
    return '11';
}

/** @param numeric-string $num */
function use_numeric(string $num): void
{
    echo 'num = ' . $num;
}

$a = test();
$b = test();
$c = test();

if (!is_numeric($a) || !is_numeric($b) || !is_numeric($c)) {
    exit('expected all numeric!');
}

use_numeric($a);
use_numeric($b);
use_numeric($c);
