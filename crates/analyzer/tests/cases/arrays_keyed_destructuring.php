<?php

declare(strict_types=1);

/**
 * @param array{name: string, age: int} $person
 */
function describe(array $person): string
{
    ['name' => $n, 'age' => $a] = $person;
    return $n . ' ' . (string) $a;
}
