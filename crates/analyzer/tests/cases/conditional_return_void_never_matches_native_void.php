<?php

declare(strict_types=1);

/**
 * A conditional return type expands to the union of both branches, and a `never` branch contributes
 * nothing to that union, so this stays compatible with the native `void` declaration and `return`
 * still needs no value.
 *
 * @return ($c is true ? void : never)
 */
function condVoidNeverMatchesNativeVoid(bool $c): void
{
    if (!$c) {
        exit;
    }

    return;
}

/**
 * @return ($c is true ? never : void)
 */
function condNeverVoidMatchesNativeVoid(bool $c): void
{
    if ($c) {
        exit;
    }

    return;
}

/**
 * @return ($c is not true ? never : void)
 */
function condNegatedNeverVoidMatchesNativeVoid(bool $c): void
{
    if (!$c) {
        exit;
    }

    return;
}
