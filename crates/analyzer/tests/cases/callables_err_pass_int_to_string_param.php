<?php

declare(strict_types=1);

function callables_takes_str_named(string $value): int
{
    return strlen($value);
}

/** @mago-expect analysis:invalid-argument */
callables_takes_str_named(value: 42);
