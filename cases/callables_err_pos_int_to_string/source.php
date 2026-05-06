<?php

declare(strict_types=1);

function callables_str_target(string $s): int
{
    return strlen($s);
}

callables_str_target(99);
