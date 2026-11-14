<?php

declare(strict_types=1);

use RuntimeException;

final class Box
{
    public ?string $name = null;
}

/** @throws RuntimeException */
function coalesce_keeps_empty_string(?string $value): string
{
    $value ?? throw new RuntimeException('missing');

    if ($value === '') {
        return 'empty';
    }

    return $value;
}

/** @throws RuntimeException */
function coalesce_keeps_false(?bool $flag): bool
{
    $flag ?? throw new RuntimeException('missing');

    return $flag;
}

/** @throws RuntimeException */
function coalesce_property(Box $box): string
{
    $box->name ?? throw new RuntimeException('missing');

    return $box->name;
}

/** @throws RuntimeException */
function or_throw(?string $value): string
{
    $value !== null || throw new RuntimeException('missing');

    return $value;
}

/** @throws RuntimeException */
function and_throw(?string $value): string
{
    $value === null && throw new RuntimeException('missing');

    return $value;
}

function or_exit(string|false $input): string
{
    ($line = $input) !== false or exit(1);

    return $line;
}

function exit_is_an_effect(): void
{
    defined('SOME_CONSTANT') || exit(1);
}
