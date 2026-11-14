<?php

declare(strict_types=1);

function nonnull_mixed_redundant_not_null(mixed $x): string
{
    // @mago-expect analysis:redundant-comparison,redundant-logical-operation
    if (isset($x) && $x !== null) {
        return 'set';
    }

    return '';
}

function nonnull_mixed_impossible_null(mixed $x): string
{
    if (isset($x)) {
        // @mago-expect analysis:redundant-comparison,impossible-condition
        if ($x === null) {
            return 'null';
        }
    }

    return '';
}

function nonnull_mixed_takes_string(string $value): string
{
    return $value;
}

function nonnull_mixed_null_branch_is_never(mixed $x): string
{
    if ($x !== null) {
        // @mago-expect analysis:redundant-comparison,impossible-condition
        if (null === $x) {
            // @mago-expect analysis:no-value
            return nonnull_mixed_takes_string($x);
        }
    }

    return '';
}

function mixed_null_check_is_kept(mixed $x): string
{
    if ($x === null) {
        return 'null';
    }

    return 'set';
}
