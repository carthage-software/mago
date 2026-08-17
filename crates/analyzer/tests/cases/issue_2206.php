<?php

declare(strict_types=1);

function test_incorrect(mixed $value): mixed
{
    if (is_array($value)) {
        return $value;
    }

    if (is_scalar($value)) {
        return $value;
    }

    return null;
}

function test_correct(mixed $value): mixed
{
    if (is_array($value)) {
        return $value;
    }

    if (is_string($value)) {
        return $value;
    }

    if (is_scalar($value)) {
        return $value;
    }

    return null;
}
