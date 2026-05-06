<?php

declare(strict_types=1);

function takesBool(bool $b): bool
{
    return $b;
}

$a = 1 === '1';
takesBool($a);
