<?php

declare(strict_types=1);

/**
 * @param array<string, mixed> $input
 */
function closure_reference_capture_defines_variable_for_later_arguments(array $input): mixed
{
    array_walk_recursive(
        $input,
        static function (mixed $value) use (&$collected): void {
            echo (string) $value;
        },
        $collected,
    );

    return $collected;
}
