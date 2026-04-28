<?php

declare(strict_types=1);

function callables_three_params(string $a, int $b, bool $c): string
{
    return $a . $b . ($c ? 't' : 'f');
}

/** @mago-expect analysis:too-few-arguments */
callables_three_params(a: 'x');
