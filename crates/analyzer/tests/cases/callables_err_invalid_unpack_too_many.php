<?php

declare(strict_types=1);

function callables_one_arg(string $s): string
{
    return $s;
}

/** @mago-expect analysis:too-many-arguments */
callables_one_arg(...['x', 'y']);
