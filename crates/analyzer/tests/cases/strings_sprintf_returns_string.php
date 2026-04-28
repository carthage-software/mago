<?php

declare(strict_types=1);

function probe(string $name, int $age): string
{
    return sprintf('Name: %s, Age: %d', $name, $age);
}
