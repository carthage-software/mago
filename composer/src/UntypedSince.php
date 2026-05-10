<?php

declare(strict_types=1);

namespace Mago;

use Attribute;

/**
 * Treats the parameter, return, or property as having no native type
 * declaration starting from the given PHP version.
 *
 * Argument MUST be a literal integer.
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
final class UntypedSince
{
    /**
     * @no-named-arguments
     */
    public function __construct(
        public int $version,
    ) {}
}
