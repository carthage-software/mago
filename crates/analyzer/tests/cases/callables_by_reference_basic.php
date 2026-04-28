<?php

declare(strict_types=1);

function callables_increment(int &$n): void
{
    $n += 1;
}

$value = 0;
callables_increment($value);
callables_increment($value);
echo $value;
