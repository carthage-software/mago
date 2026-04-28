<?php

declare(strict_types=1);

function callables_required_required(string $a, int $b): string
{
    return $a . $b;
}

/** @mago-expect analysis:too-few-arguments */
callables_required_required(b: 5);
