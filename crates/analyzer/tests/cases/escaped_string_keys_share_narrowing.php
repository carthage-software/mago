<?php

declare(strict_types=1);

/**
 * @param array{'p\q': int|null, plain: int|null} $values
 */
function same_key_written_differently(array $values): int
{
    if ($values["p\\q"] !== null) {
        return $values['p\q'];
    }

    if ($values["plain"] !== null) {
        return $values['plain'];
    }

    return 0;
}

/**
 * @param array<string, int|null> $values
 */
function different_keys(array $values): int
{
    if ($values["a\tb"] !== null) { // @mago-expect analysis:possibly-undefined-string-array-index
        return $values['a\tb']; // @mago-expect analysis:nullable-return-statement,invalid-return-statement
    }

    return 0;
}
