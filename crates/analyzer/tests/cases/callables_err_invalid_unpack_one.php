<?php

declare(strict_types=1);

function callables_strs_two(string $a, string $b): string
{
    return $a . $b;
}

/** @mago-expect analysis:invalid-argument */
callables_strs_two(...['a', 1]);
