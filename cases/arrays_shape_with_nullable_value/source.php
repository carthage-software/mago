<?php

declare(strict_types=1);

/**
 * @return array{name: string, age: int|null}
 */
function maybe_age(): array
{
    return ['name' => 'Bob', 'age' => null];
}
