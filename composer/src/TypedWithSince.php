<?php

declare(strict_types=1);

namespace Mago;

use Attribute;

/**
 * Overrides the declared type of a parameter, return, or property starting
 * from the given PHP version. The type string is parsed by Mago in the same
 * way as a docblock type.
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
final class TypedWithSince
{
    /**
     * @no-named-arguments
     */
    public function __construct(
        public string $type,
        public int $version,
    ) {}
}
