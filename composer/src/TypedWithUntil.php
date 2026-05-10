<?php

declare(strict_types=1);

namespace Mago;

use Attribute;

/**
 * Overrides the declared type of a parameter, return, or property up to and
 * including the given PHP version.
 *
 * Both arguments MUST be literal values.
 *
 * @api
 */
#[Attribute(
    Attribute::TARGET_PARAMETER
    | Attribute::TARGET_FUNCTION
    | Attribute::TARGET_METHOD
    | Attribute::TARGET_PROPERTY
    | Attribute::IS_REPEATABLE,
)]
final class TypedWithUntil
{
    /**
     * @no-named-arguments
     */
    public function __construct(
        public string $type,
        public int $version,
    ) {}
}
