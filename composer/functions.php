<?php

declare(strict_types=1);

namespace Mago;

/**
 * Inspect the type of the given value.
 *
 * This function is used for debugging purposes to output the type of a variable.
 *
 * @param mixed ...$value The value(s) whose type(s) will be dumped.
 *
 * @return void This function does not return a value.
 */
function inspect(mixed ...$value): void
{
}

/**
 * Confirms that the given value is of the specified type statically.
 *
 * This function is used to ensure that the value conforms to the expected type
 * during static analysis. It does not perform any runtime checks or throw exceptions.
 *
 * @param mixed $value The value to check.
 * @param literal-string $type The expected type of the value.
 *
 * @return void This function does not return a value.
 */
function confirm(mixed $value, string $type): void
{
}
