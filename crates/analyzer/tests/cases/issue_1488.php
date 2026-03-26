<?php

declare(strict_types=1);

function take_int(int $v): void
{
    echo $v;
}

$mon = (int) date('n', time());

if ($mon === date('n', time())) { // @mago-expect analysis:redundant-comparison,impossible-condition - expected, int !== string
    take_int($mon); // @mago-expect analysis:no-value - expected
}

if ($mon == date('n', time())) {
    take_int($mon);
}
