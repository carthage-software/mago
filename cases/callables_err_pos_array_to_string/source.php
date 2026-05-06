<?php

declare(strict_types=1);

function callables_str_target_two(string $s): int
{
    return strlen($s);
}

callables_str_target_two([1, 2]);
