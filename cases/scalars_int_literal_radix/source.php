<?php

declare(strict_types=1);

function takesInt(int $n): int
{
    return $n;
}

$a = 0xff;
$b = 0o17;
$c = 017;
$d = 0b1010;

takesInt($a);
takesInt($b);
takesInt($c);
takesInt($d);
