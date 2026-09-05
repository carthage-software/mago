<?php

declare(strict_types=1);

/** @param class-string $class */
function takes_class_string(string $class): void
{
    echo $class;
}

function narrows_disjunction(string $class): void
{
    if (class_exists($class) || interface_exists($class)) {
        takes_class_string($class);
    }

    if (interface_exists($class) || class_exists($class)) {
        takes_class_string($class);
    }
}

function narrows_after_early_return(string $class): void
{
    if (!class_exists($class) && !interface_exists($class)) {
        return;
    }

    takes_class_string($class);
}

function does_not_narrow_unchecked_alternative(string $class, bool $flag): void
{
    if (class_exists($class) || $flag) {
        /** @mago-expect analysis:possibly-invalid-argument */
        takes_class_string($class);
    }
}

/** @param class-string $class */
function preserves_class_string_when_checks_fail(string $class): void
{
    if (!class_exists($class) && !interface_exists($class)) {
        takes_class_string($class);
    }
}
