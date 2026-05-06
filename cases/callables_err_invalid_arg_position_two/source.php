<?php

declare(strict_types=1);

function callables_pos_two(string $a, int $b): string
{
    return $a . $b;
}

callables_pos_two('hello', 'not int');
