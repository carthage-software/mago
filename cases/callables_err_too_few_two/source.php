<?php

declare(strict_types=1);

function callables_two_required_two(string $a, int $b): string
{
    return $a . $b;
}

callables_two_required_two('only');
