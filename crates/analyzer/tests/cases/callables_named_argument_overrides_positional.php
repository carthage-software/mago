<?php

declare(strict_types=1);

function callables_collide(string $a, int $b): string
{
    return $a . $b;
}

/** @mago-expect analysis:too-many-arguments */
/** @mago-expect analysis:named-argument-overrides-positional */
callables_collide('x', 1, a: 'y');
