<?php

declare(strict_types=1);

/** @param object{bar?: string} $foo */
function shape_isset_in_branch(object $foo): void
{
    if (isset($foo->bar)) {
        shape_requires_bar($foo);
    }
}

/** @param object{bar?: string} $foo */
function shape_isset_after_early_return(object $foo): void
{
    if (!isset($foo->bar)) {
        return;
    }

    shape_requires_bar($foo);
}

/** @param object{bar: ?string} $foo */
function shape_not_null(object $foo): void
{
    if ($foo->bar !== null) {
        shape_requires_bar($foo);
    }
}

/** @param object{bar?: object{baz?: int}} $foo */
function shape_nested_isset(object $foo): void
{
    if (isset($foo->bar->baz)) {
        shape_requires_nested($foo);
    }
}

/** @param object{bar?: string, baz: int} $foo */
function shape_property_exists(object $foo): int
{
    if (property_exists($foo, 'bar')) {
        return $foo->baz;
    }

    return 0;
}

/** @param object{bar: string} $foo */
function shape_requires_bar(object $foo): void
{
    echo $foo->bar;
}

/** @param object{bar: object{baz: int}} $foo */
function shape_requires_nested(object $foo): void
{
    echo $foo->bar->baz;
}
