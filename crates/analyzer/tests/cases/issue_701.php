<?php

/**
 * Tests that checking `$value === []` on a nonnull type doesn't produce
 * a false positive "redundant condition" error.
 *
 * @see https://github.com/carthage-software/mago/issues/701
 */

function validate(mixed $value): void
{
    if ($value === null || $value === []) {
        return;
    }
}

function validate_with_negation(mixed $value): void
{
    if ($value !== null && $value !== []) {
        // $value is nonnull and non-empty-countable
    }
}
