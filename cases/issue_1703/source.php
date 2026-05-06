<?php

declare(strict_types=1);

function f(mixed $v): bool
{
    $b = $v !== null && $v !== [];

    return $b;
}
