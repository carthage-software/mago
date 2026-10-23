<?php

declare(strict_types=1);

/** @param ArrayObject<int, int|string>&ArrayAccess<int, covariant int> $values */
function readProjectedOffset(ArrayObject $values): void
{
    $value = $values[0] ?? null;
    if ($value !== null) {
        consumeOffsetInt($value);
    }
}

/** @param ArrayAccess<int, int>&ArrayAccess<string, string> $values */
function readDistinctKeyDomains(ArrayAccess $values): void
{
    $byInt = $values[0] ?? null;
    if ($byInt !== null) {
        consumeOffsetInt($byInt);
        /** @mago-expect analysis:invalid-argument */
        consumeOffsetString($byInt);
    }
    $byString = $values['name'] ?? null;
    if ($byString !== null) {
        consumeOffsetString($byString);
        /** @mago-expect analysis:invalid-argument */
        consumeOffsetInt($byString);
    }
}

/** @param ArrayObject<int, int|string>&ArrayAccess<int, covariant int> $values */
function writeProjectedOffset(ArrayObject $values): void
{
    $values[0] = 'updated';
    /** @mago-expect analysis:invalid-array-access-assignment-value */
    $values[1] = true;
}

/** @param ArrayObject<int|string, int|string>&ArrayAccess<int, covariant int> $values */
function readPartiallyProjectedOffset(ArrayObject $values, int|string $key): void
{
    $value = $values[$key] ?? null;
    if ($value !== null) {
        /** @mago-expect analysis:possibly-invalid-argument */
        consumeOffsetInt($value);
    }
}

function consumeOffsetInt(int $value): void
{
    echo $value;
}

function consumeOffsetString(string $value): void
{
    echo $value;
}
