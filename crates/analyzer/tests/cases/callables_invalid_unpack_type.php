<?php

declare(strict_types=1);

function callables_takes_strings(string $a, string $b): string
{
    return $a . $b;
}

/** @mago-expect analysis:invalid-argument */
callables_takes_strings(...['a', 1]);
