<?php

declare(strict_types=1);

function callables_int_named(int $count): int
{
    return $count;
}

callables_int_named(count: 'five');
