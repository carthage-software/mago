<?php

declare(strict_types=1);

/** @param array{name: string, age: int} $person */
function greet(array $person): string
{
    return 'Hi ' . $person['name'];
}

function caller(): void
{
    // @mago-expect analysis:possibly-invalid-argument
    greet(['name' => 'Alice']);
}
