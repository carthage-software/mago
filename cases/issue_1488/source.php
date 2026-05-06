<?php

declare(strict_types=1);

function take_int(int $v): void
{
    echo $v;
}

$mon = (int) date('n', time());

if ($mon === date('n', time())) {
    take_int($mon);
}

if ($mon == date('n', time())) {
    take_int($mon);
}
