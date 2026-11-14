<?php

declare(strict_types=1);

/** @param object{bar?: string} $foo */
function after_isset(object $foo): void
{
    if (isset($foo->bar)) {
        unset($foo->bar);
        /** @mago-expect analysis:possibly-invalid-argument */
        requires_bar($foo);
    }
}

/** @param object{bar: string} $foo */
function without_isset(object $foo): void
{
    unset($foo->bar);
    /** @mago-expect analysis:possibly-invalid-argument */
    requires_bar($foo);
}

/** @param object{bar?: object{baz?: string}} $foo */
function nested(object $foo): void
{
    if (isset($foo->bar->baz)) {
        unset($foo->bar->baz);
        /** @mago-expect analysis:possibly-invalid-argument */
        requires_baz($foo);
    }
}

/** @param object{bar: string} $foo */
function requires_bar(object $foo): string
{
    return $foo->bar;
}

/** @param object{bar: object{baz: string}} $foo */
function requires_baz(object $foo): string
{
    return $foo->bar->baz;
}
