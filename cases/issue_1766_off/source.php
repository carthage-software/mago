<?php

declare(strict_types=1);

/**
 * @return non-empty-list<string>
 */
function tokenize(string $input): array
{
    return explode(' ', $input);
}

function destructure_default(string $input): void
{
    [$head, $tail] = tokenize($input);

    echo $head . ' / ' . $tail . "\n";
}

function read_default(string $key): void
{
    /** @var array<string, int> $counters */
    $counters = ['hits' => 1, 'miss' => 2];

    $value = $counters[$key];

    echo $value . "\n";
}
