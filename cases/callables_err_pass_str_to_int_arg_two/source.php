<?php

declare(strict_types=1);

function callables_take_int_a(int $a): int
{
    return $a;
}

callables_take_int_a('not int');
