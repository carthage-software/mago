<?php

declare(strict_types=1);

final class Letter
{
    public function __toString(): string
    {
        return 'a';
    }
}

final class Plain
{
}

function object_or_string(Letter|string $value): bool
{
    if ($value == 'a' && $value instanceof Letter) {
        return true;
    }

    return false;
}

function stringable(Stringable $value): string
{
    if ($value == 'a') {
        return (string) $value;
    }

    return '';
}

function plain_or_string(Plain|string $value): string
{
    if ($value == 'a') {
        return $value;
    }

    return '';
}
