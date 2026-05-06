<?php

declare(strict_types=1);

/** @param array{name: string, age?: int} $p */
function describeCC(array $p): string
{
    $age = $p['age'] ?? 0;

    return $p['name'] . ':' . $age;
}

echo describeCC(['name' => 'alice']);
echo describeCC(['name' => 'bob', 'age' => 30]);
