<?php

declare(strict_types=1);

namespace Issue2411;

function take_string(string $_s): void {}

/**
 * @param array<string, string> $files
 * @param list<string> $keys
 */
function test(array $keys, array $files): int
{
    foreach ($keys as $v) {
        $got_it = true;
        if (!isset($files[$v])) {
            $got_it = false;
        } else {
        }

        if ($got_it) {
            take_string($files[$v]);

            return 0;
        }
    }

    return -1;
}

/**
 * @param array<string, string> $files
 * @param list<string> $keys
 */
function test2(array $keys, array $files): int
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
    $valid = true;
    if ($value === null) {
        $valid = false;
    }

    if ($valid) {
        take_string($value);
    }
}

function inverseGuard(?string $value): void
{
    $missing = false;
    if ($value === null) {
        $missing = true;
    }

    if (!$missing) {
        take_string($value);
    }
}

function assignedOnlyInElse(?string $value): void
{
    $valid = false;
    if ($value === null) {
    } else {
        $valid = true;
    }

    if ($valid) {
        take_string($value);
    }
}

function unknownInitialFlag(?string $value, bool $valid): void
{
    if ($value !== null) {
        $valid = true;
    }

    if ($valid) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}

function overwrittenFlag(?string $value, bool $replacement): void
{
    $valid = true;
    if ($value === null) {
        $valid = false;
    }

    $valid = $replacement;
    if ($valid) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}

function overwrittenValue(?string $value, ?string $replacement): void
{
    $valid = true;
    if ($value === null) {
        $valid = false;
    }

    $value = $replacement;
    if ($valid) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}

/** @param-out true $flag */
function setTrue(bool &$flag): void
{
    $flag = true;
}

function changedByReference(?string $value): void
{
    $valid = false;
    if ($value === null) {
        setTrue($valid);
    } else {
        $valid = true;
    }

    // @mago-expect analysis:redundant-condition
    if ($valid) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}

function anotherFalseBranch(?string $value, bool $skip): void
{
    $missing = false;
    if ($value !== null) {
        $missing = true;
    } elseif ($skip) {
        $missing = true;
    }

    if ($missing) {
        // @mago-expect analysis:possibly-null-argument
        take_string($value);
    }
}

function changedInsideBranch(?string $value): void
{
    $valid = true;
    if ($value === null) {
        $valid = false;
    } else {
        $value = null;
    }

    if ($valid) {
        // @mago-expect analysis:null-argument
        take_string($value);
    }
}

/** @param array<string, string> $files */
function unsetInsideBranch(array $files, string $key): void
{
    $valid = false;
    if (isset($files[$key])) {
        unset($files[$key]);
        $valid = true;
    }

    if ($valid) {
        // @mago-expect analysis:possibly-undefined-string-array-index,possibly-null-argument
        take_string($files[$key]);
    }
}

/** @param array<string, string> $files */
function changedKeyInsideBranch(array $files, string $key, string $replacement): void
{
    $valid = false;
    if (isset($files[$key])) {
        $key = $replacement;
        $valid = true;
    }

    if ($valid) {
        // @mago-expect analysis:possibly-undefined-string-array-index,possibly-null-argument
        take_string($files[$key]);
    }
}
