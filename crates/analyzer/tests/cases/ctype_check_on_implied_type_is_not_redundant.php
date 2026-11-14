<?php

declare(strict_types=1);

function is_negative(int $id): bool
{
    $s = (string) $id;

    if (!ctype_digit($s)) {
        return true;
    }

    return false;
}

/** @param non-empty-lowercase-string $s */
function letters_only(string $s): int
{
    if (ctype_lower($s)) {
        return 1;
    } else {
        return 2;
    }
}

/** @param '-5'|'12' $s */
function literal(string $s): string
{
    if (!ctype_digit($s)) {
        return $s;
    }

    return 'digits';
}

/** @return numeric-string|null */
function digits(string $s): ?string
{
    if (ctype_digit($s)) {
        return $s;
    }

    return null;
}
