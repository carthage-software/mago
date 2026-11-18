<?php

final class ArrayMapCallbackFailure extends Exception
{
}

final class ArrayMapArgumentFailure extends Exception
{
}

/**
 * @throws ArrayMapCallbackFailure
 */
function array_map_callback_empty_single_array(): void
{
    array_map(
        callback: /** @throws ArrayMapCallbackFailure */
        static function (mixed $value): int {
            throw new ArrayMapCallbackFailure();
        },
        array: [],
    );
}

/**
 * @throws ArrayMapCallbackFailure
 */
function array_map_callback_empty_multiple_arrays(): void
{
    array_map(
        /** @throws ArrayMapCallbackFailure */
        static function (mixed $left, mixed $right): int {
            throw new ArrayMapCallbackFailure();
        },
        [],
        [],
    );
}

/**
 * @throws ArrayMapCallbackFailure
 */
function array_map_callback_one_empty_array(): void
{
    array_map(
        /** @throws ArrayMapCallbackFailure */
        static function (mixed $left, mixed $right): int {
            throw new ArrayMapCallbackFailure();
        },
        [],
        [1],
    );
}

/**
 * @param array $values
 * @throws ArrayMapCallbackFailure
 */
function array_map_callback_unknown_array(array $values): void
{
    array_map(
        /** @throws ArrayMapCallbackFailure */
        static function (mixed $value): int {
            throw new ArrayMapCallbackFailure();
        },
        $values,
    );
}

function array_map_callback_empty_without_contract(): void
{
    array_map(
        /** @throws ArrayMapCallbackFailure */
        static function (mixed $value): int {
            throw new ArrayMapCallbackFailure();
        },
        [],
    );
}

/**
 * @return array{}
 * @throws ArrayMapArgumentFailure
 */
function array_map_callback_empty_input_that_throws(): array
{
    throw new ArrayMapArgumentFailure();
}

/**
 * @throws ArrayMapArgumentFailure
 */
function array_map_callback_empty_preserves_argument_throw(): void
{
    array_map(
        /** @throws ArrayMapCallbackFailure */
        static function (mixed $value): int {
            throw new ArrayMapCallbackFailure();
        },
        array_map_callback_empty_input_that_throws(),
    );
}
