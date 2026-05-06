<?php

declare(strict_types=1);

function callables_has_int_named(int $value): int
{
    return $value;
}

callables_has_int_named(value: 'wrong');
