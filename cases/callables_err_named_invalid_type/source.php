<?php

declare(strict_types=1);

function callables_named_int(int $value): int
{
    return $value;
}

callables_named_int(value: 'oops');
