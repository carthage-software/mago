<?php

declare(strict_types=1);

final class Box
{
    public ?string $value = null;
}

function take_string(string $value): string
{
    return $value;
}

function not_null(?Box $box): string
{
    if ($box?->value !== null) {
        return take_string($box?->value);
    }

    return '';
}

function yoda_not_null(?Box $box): string
{
    if (null !== $box?->value) {
        return take_string($box?->value);
    }

    return '';
}

function truthy(?Box $box): string
{
    if ($box?->value) {
        return take_string($box?->value);
    }

    return '';
}
