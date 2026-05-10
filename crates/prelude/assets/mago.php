<?php

declare(strict_types=1);

namespace Mago;

use Attribute;

/**
 * Marks the symbol as available starting from the given PHP version.
 *
 * Mago treats the symbol as non-existent on older versions, so references
 * to it report as undefined when the configured PHP version is below `$version`.
 *
 * Use the `PHP_VERSION_ID` style: e.g. `80100` for `8.1.0`, `80299` for `8.2.99`.
 *
 * Argument MUST be a literal integer.
 */
#[Attribute(
    Attribute::TARGET_CLASS
        | Attribute::TARGET_FUNCTION
        | Attribute::TARGET_METHOD
        | Attribute::TARGET_PROPERTY
        | Attribute::TARGET_CLASS_CONSTANT
        | Attribute::TARGET_CONSTANT
        | Attribute::TARGET_PARAMETER
        | Attribute::IS_REPEATABLE,
)]
final class AvailableSince
{
    /**
     * @no-named-arguments
     */
    public function __construct(public int $version) {}
}

/**
 * Marks the symbol as available up to and including the given PHP version.
 *
 * Mago treats the symbol as non-existent on newer versions, so references
 * to it report as undefined when the configured PHP version is above `$version`.
 *
 * Argument MUST be a literal integer.
 */
#[Attribute(
    Attribute::TARGET_CLASS
        | Attribute::TARGET_FUNCTION
        | Attribute::TARGET_METHOD
        | Attribute::TARGET_PROPERTY
        | Attribute::TARGET_CLASS_CONSTANT
        | Attribute::TARGET_CONSTANT
        | Attribute::TARGET_PARAMETER
        | Attribute::IS_REPEATABLE,
)]
final class AvailableUntil
{
    /**
     * @no-named-arguments
     */
    public function __construct(public int $version) {}
}

/**
 * Marks the parameter as optional starting from the given PHP version.
 *
 * Argument MUST be a literal integer.
 */
#[Attribute(Attribute::TARGET_PARAMETER | Attribute::IS_REPEATABLE)]
final class OptionalSince
{
    /**
     * @no-named-arguments
     */
    public function __construct(public int $version) {}
}

/**
 * Marks the parameter as optional up to and including the given PHP version.
 *
 * Argument MUST be a literal integer.
 */
#[Attribute(Attribute::TARGET_PARAMETER | Attribute::IS_REPEATABLE)]
final class OptionalUntil
{
    /**
     * @no-named-arguments
     */
    public function __construct(public int $version) {}
}

/**
 * Marks the parameter as required starting from the given PHP version.
 *
 * Argument MUST be a literal integer.
 */
#[Attribute(Attribute::TARGET_PARAMETER | Attribute::IS_REPEATABLE)]
final class RequiredSince
{
    /**
     * @no-named-arguments
     */
    public function __construct(public int $version) {}
}

/**
 * Marks the parameter as required up to and including the given PHP version.
 *
 * Argument MUST be a literal integer.
 */
#[Attribute(Attribute::TARGET_PARAMETER | Attribute::IS_REPEATABLE)]
final class RequiredUntil
{
    /**
     * @no-named-arguments
     */
    public function __construct(public int $version) {}
}

/**
 * Overrides the declared type of a parameter, return, or property starting
 * from the given PHP version. The type string is parsed by Mago in the same
 * way as a docblock type.
 *
 * Both arguments MUST be literal values.
 */
#[Attribute(
    Attribute::TARGET_PARAMETER | Attribute::TARGET_FUNCTION | Attribute::TARGET_METHOD | Attribute::TARGET_PROPERTY | Attribute::IS_REPEATABLE,
)]
final class TypedWithSince
{
    /**
     * @no-named-arguments
     */
    public function __construct(public string $type, public int $version) {}
}

/**
 * Overrides the declared type of a parameter, return, or property up to and
 * including the given PHP version.
 *
 * Both arguments MUST be literal values.
 */
#[Attribute(
    Attribute::TARGET_PARAMETER | Attribute::TARGET_FUNCTION | Attribute::TARGET_METHOD | Attribute::TARGET_PROPERTY | Attribute::IS_REPEATABLE,
)]
final class TypedWithUntil
{
    /**
     * @no-named-arguments
     */
    public function __construct(public string $type, public int $version) {}
}

/**
 * Treats the parameter, return, or property as having no native type
 * declaration starting from the given PHP version.
 *
 * Argument MUST be a literal integer.
 */
#[Attribute(
    Attribute::TARGET_PARAMETER | Attribute::TARGET_FUNCTION | Attribute::TARGET_METHOD | Attribute::TARGET_PROPERTY | Attribute::IS_REPEATABLE,
)]
final class UntypedSince
{
    /**
     * @no-named-arguments
     */
    public function __construct(public int $version) {}
}

/**
 * Treats the parameter, return, or property as having no native type
 * declaration up to and including the given PHP version.
 *
 * Argument MUST be a literal integer.
 */
#[Attribute(
    Attribute::TARGET_PARAMETER | Attribute::TARGET_FUNCTION | Attribute::TARGET_METHOD | Attribute::TARGET_PROPERTY | Attribute::IS_REPEATABLE,
)]
final class UntypedUntil
{
    /**
     * @no-named-arguments
     */
    public function __construct(public int $version) {}
}

/**
 * Inspect the type of the given value.
 *
 * This function is used for debugging purposes to output the type of a variable.
 *
 * @param mixed ...$value The value(s) whose type(s) will be dumped.
 *
 * @return void This function does not return a value.
 */
function inspect(mixed ...$value): void {}

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
function confirm(mixed $value, string $type): void {}
