<?php

declare(strict_types=1);

/**
 * @return array{user: array{name: string, age: int}}
 */
function nested(): array
{
    return ['user' => ['name' => 'Alice', 'age' => 30]];
}
