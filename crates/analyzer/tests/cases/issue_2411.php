<?php

declare(strict_types=1);

namespace Issue2411;

function take_string(string $_s): void {}

/**
 * @param list<string> $keys
 * @param array<string, string> $files
 */
function initializedBeforeIf(array $keys, array $files): int
{
    foreach ($keys as $v) {
        $got_it = true;
        if (!isset($files[$v])) {
            $got_it = false;
        } else {
            // Keep the initial value.
        }

        if ($got_it) {
            take_string($files[$v]);
            return 0;
        }
    }

    return -1;
}

/**
 * @param list<string> $keys
 * @param array<string, string> $files
 */
function assignedInBothBranches(array $keys, array $files): int
{
    foreach ($keys as $v) {
        if (!isset($files[$v])) {
            $got_it = false;
        } else {
            $got_it = true;
        }

        if ($got_it) {
            take_string($files[$v]);
            return 0;
        }
    }

    return -1;
}

function withoutElse(?string $value): void
{
    $hasValue = true;
    if ($value === null) {
        $hasValue = false;
    }

    if ($hasValue) {
        take_string($value);
    }
}

function initializedFalse(?string $value): void
{
    $hasValue = false;
    if ($value !== null) {
        $hasValue = true;
    }

    if ($hasValue) {
        take_string($value);
    }
}

function negatedFlag(?string $value): void
{
    $missing = false;
    if ($value === null) {
        $missing = true;
    }

    if (!$missing) {
        take_string($value);
    }
}

function reassignedFlag(?string $value, bool $override): void
{
    $hasValue = true;
    if ($value === null) {
        $hasValue = false;
    }

    $hasValue = $override;
    if ($hasValue) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}

function reassignedValue(?string $value, ?string $replacement): void
{
    $hasValue = true;
    if ($value === null) {
        $hasValue = false;
    }

    $value = $replacement;
    if ($hasValue) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}

function reassignedValueInElse(?string $value, ?string $replacement): void
{
    $hasValue = true;
    if ($value === null) {
        $hasValue = false;
    } else {
        $value = $replacement;
    }

    if ($hasValue) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}

function reassignedValueInIf(?string $value, ?string $replacement): void
{
    $hasValue = false;
    if ($value !== null) {
        $hasValue = true;
        $value = $replacement;
    }

    if ($hasValue) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}
