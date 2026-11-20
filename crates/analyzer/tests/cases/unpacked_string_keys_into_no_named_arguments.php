<?php

declare(strict_types=1);

/**
 * @no-named-arguments
 */
function no_named_arguments_target(int $a, string $b): void
{
    echo $a, $b;
}

/**
 * @param array{a: int, b: string} $named
 * @param list{int, string} $positional
 */
function unpack_into_no_named_arguments(array $named, array $positional): void
{
    /** @mago-expect analysis:named-argument-not-allowed */
    no_named_arguments_target(...$named);
    no_named_arguments_target(...$positional);
}
