<?php

declare(strict_types=1);

namespace BooleanNarrowing;

function takesFalse(false $value): void {}

function takesTrue(true $value): void {}

function takesFalseOrInt(false|int $value): void {}

function takesFalseOrString(false|string $value): void {}

function takesTrueOrInt(true|int $value): void {}

function intUnion(bool|int $value): void
{
    if ($value !== true) {
        takesFalseOrInt($value);
    } else {
        takesTrue($value);
    }

    if (true !== $value) {
        takesFalseOrInt($value);
    } else {
        takesTrue($value);
    }

    if ($value !== false) {
        takesTrueOrInt($value);
    } else {
        takesFalse($value);
    }
}

function stringUnion(bool|string $value): void
{
    if (true !== $value) {
        takesFalseOrString($value);
    } else {
        takesTrue($value);
    }
}

function negatedComparison(bool|int $value): void
{
    if (!($value === true)) {
        takesFalseOrInt($value);
    } else {
        takesTrue($value);
    }
}

function assertedBoolean(bool|int $value): void
{
    if (!$value && is_bool($value)) {
        takesFalse($value);
    }

    if ($value && is_bool($value)) {
        takesTrue($value);
    }
}

/**
 * @assert bool $value
 * @throws \InvalidArgumentException
 */
function requireBoolean(mixed $value): void
{
    if (!is_bool($value)) {
        throw new \InvalidArgumentException();
    }
}

/** @throws \InvalidArgumentException */
function assertedLiteral(bool|int $value): void
{
    if (!$value) {
        requireBoolean($value);
        takesFalse($value);
    } else {
        requireBoolean($value);
        takesTrue($value);
    }
}
