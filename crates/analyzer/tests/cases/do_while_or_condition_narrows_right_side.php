<?php

declare(strict_types=1);

function next_value(): ?string
{
    return rand(0, 1) === 1 ? 'value' : null;
}

function read(): string
{
    do {
        $b = next_value();
    } while ($b === null || strlen($b) > 4);

    return $b;
}

function read_with_low_or(): string
{
    do {
        $b = next_value();
    } while ($b === null or strlen($b) > 4);

    return $b;
}

function read_with_and(): ?string
{
    do {
        $b = next_value();
    } while ($b !== null && strlen($b) > 4);

    return $b;
}
