<?php

declare(strict_types=1);

namespace Mago;

use Attribute;

/**
 * Marks the symbol as available up to and including the given PHP version.
 *
 * Mago treats the symbol as non-existent on newer versions, so references
 * to it report as undefined when the configured PHP version is above `$version`.
 *
 * Argument MUST be a literal integer.
 *
 * @api
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
    public function __construct(
        public int $version,
    ) {}
}
