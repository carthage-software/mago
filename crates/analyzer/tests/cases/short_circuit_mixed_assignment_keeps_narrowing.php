<?php

declare(strict_types=1);

/** @param array<mixed> $arguments */
function after_or(array $arguments, bool $skip): string
{
    // @mago-expect analysis:mixed-assignment
    if ($skip || null === ($name = $arguments[0] ?? null)) {
        exit(1);
    }

    // @mago-expect analysis:redundant-null-coalesce
    return get_debug_type($name ?? null);
}

/** @param array<mixed> $arguments */
function inside_and(array $arguments, bool $ok): string
{
    // @mago-expect analysis:mixed-assignment
    if ($ok && null !== ($name = $arguments[0] ?? null)) {
        // @mago-expect analysis:redundant-null-coalesce
        return get_debug_type($name ?? null);
    }

    exit(1);
}
