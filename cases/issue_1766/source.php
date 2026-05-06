<?php

declare(strict_types=1);

/**
 * @return non-empty-list<string>
 */
function tokenize(string $input): array
{
    return explode(' ', $input);
}

function destructure_possibly_missing(string $input): void
{
    [$head, $tail] = tokenize($input);

    if (null === $tail) {
        echo "single token\n";
        return;
    }

    echo $head . ' / ' . $tail . "\n";
}

function read_unknown_key(string $key): void
{
    /** @var array<string, int> $counters */
    $counters = ['hits' => 1, 'miss' => 2];

    $value = $counters[$key];

    if (null === $value) {
        echo "missing\n";
        return;
    }

    echo $value . "\n";
}
