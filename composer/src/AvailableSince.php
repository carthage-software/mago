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
final class AvailableSince
{
    /**
     * @no-named-arguments
     */
    public function __construct(
        public int $version,
    ) {}
}
