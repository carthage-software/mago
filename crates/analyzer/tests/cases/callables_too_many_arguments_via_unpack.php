<?php

declare(strict_types=1);

function callables_takes_one(string $a): string
{
    return $a;
}

/** @mago-expect analysis:too-many-arguments */
callables_takes_one(...['a', 'b', 'c']);
