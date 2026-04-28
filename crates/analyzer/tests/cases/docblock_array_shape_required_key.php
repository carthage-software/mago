<?php

declare(strict_types=1);

/** @param array{name: string, age: int} $p */
function describeCB(array $p): string
{
    return $p['name'] . ':' . $p['age'];
}

echo describeCB(['name' => 'alice', 'age' => 30]);
