<?php

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
