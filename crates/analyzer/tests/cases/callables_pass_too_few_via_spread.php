<?php

declare(strict_types=1);

function callables_two_required(string $a, int $b): string
{
    return $a . $b;
}

/** @mago-expect analysis:too-few-arguments */
callables_two_required(...['hello']);
