<?php

declare(strict_types=1);

/** @throws \InvalidArgumentException */
function doBar(): int
{
    if (rand() === 1) {
        throw new \InvalidArgumentException();
    }

    return 1;
}

/** @throws \RuntimeException */
function doBaz(): string
{
    if (rand() === 1) {
        throw new \RuntimeException();
    }

    return '';
}

try {
    // $foo is surely defined in all catch blocks because the literal 1 cannot throw.
    $foo = 1;

    // @mago-expect analysis:redundant-condition
    if (isset($foo)) {
    }

    $bar = doBar();
    $baz = doBaz();
} catch (\InvalidArgumentException $e) {
    // $foo is always defined.
    // @mago-expect analysis:redundant-condition
    if (isset($foo)) {
    }

    // $bar and $baz are never defined here.
    if (isset($bar)) {
    }
    if (isset($baz)) {
    }
} catch (\RuntimeException $e) {
    // $foo and $bar are always defined.
    // @mago-expect analysis:redundant-condition
    if (isset($foo)) {
    }
    // @mago-expect analysis:redundant-condition
    if (isset($bar)) {
    }

    // $baz is never defined here.
    if (isset($baz)) {
    }
}

function catch_union(): void
{
    try {
        $alwaysDefined = 1;
        doBar();
        $onlyDefinedAfterDoBar = 1;
        doBaz();
    } catch (\InvalidArgumentException|\RuntimeException) {
        // Every throw site is reached after this assignment.
        // @mago-expect analysis:redundant-condition
        if (isset($alwaysDefined)) {
        }

        // `doBar()` can throw before this assignment.
        if (isset($onlyDefinedAfterDoBar)) {
        }
    }
}
