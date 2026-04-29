<?php

declare(strict_types=1);

class Counter
{
    public int $count = 0;
}

$c = new Counter();
$c->count = 5;
if ($c->count === 6) {
    echo 'six';
}
