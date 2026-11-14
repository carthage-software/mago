<?php

declare(strict_types=1);

/** @param-out string $value */
function fill(?string &$value): void
{
    $value = 'filled';
}

function fresh_output(): string
{
    // @mago-expect analysis:reference-to-undefined-variable
    fill($value);

    return $value;
}

/** @param list<string> $lines */
function match_in_loop(array $lines): int
{
    $count = 0;
    foreach ($lines as $line) {
        if (preg_match('/(\d+)/', $line, $found) === 1) {
            $count += count($found);
        }
    }

    return $count;
}

function set_int(int &$value): void
{
    $value = 1;
}

function not_nullable_by_reference(): int
{
    // @mago-expect analysis:reference-to-undefined-variable
    // @mago-expect analysis:null-argument
    set_int($value);

    return $value;
}
