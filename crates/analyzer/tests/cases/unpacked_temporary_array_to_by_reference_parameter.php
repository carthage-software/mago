<?php

declare(strict_types=1);

function many(int &...$values): void
{
    foreach ($values as &$value) {
        $value++;
    }
}

function one(int &$value): void
{
    $value++;
}

function unpack_literals(): void
{
    many(...[1, 2]);
    one(...[1]);
}

function unpack_variable(): int
{
    $values = [1, 2];
    many(...$values);

    return $values[0];
}

function pass_literal(): void
{
    one(1); // @mago-expect analysis:invalid-pass-by-reference
}
