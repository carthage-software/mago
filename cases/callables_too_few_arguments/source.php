<?php

declare(strict_types=1);

function callables_needs_two(string $a, int $b): string
{
    return $a . $b;
}

callables_needs_two('hello');
