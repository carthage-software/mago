<?php

declare(strict_types=1);

final class Reader
{
    public function read(string $label): int
    {
        return strlen($label);
    }
}

function label(): string
{
    return 'a';
}

function nullsafe(?Reader $reader): int
{
    $value = new stdClass();
    $read = $reader?->read($value = label());

    // @mago-expect analysis:possibly-invalid-argument
    return strlen($value) + ($read ?? 0);
}
