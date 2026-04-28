<?php

declare(strict_types=1);

function callables_needs_two(string $a, int $b): string
{
    return $a . $b;
}

/** @mago-expect analysis:too-few-arguments */
callables_needs_two('hello');
