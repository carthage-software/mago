<?php

declare(strict_types=1);

function callables_required_required(string $a, int $b): string
{
    return $a . $b;
}

callables_required_required(b: 5);
