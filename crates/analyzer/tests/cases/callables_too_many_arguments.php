<?php

declare(strict_types=1);

function callables_takes_two(string $a, int $b): string
{
    return $a . $b;
}

/** @mago-expect analysis:too-many-arguments */
callables_takes_two('hello', 1, 2);
