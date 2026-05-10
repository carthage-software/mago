<?php

declare(strict_types=1);

namespace Mago;

use Attribute;

/**
 * Marks the parameter as required up to and including the given PHP version.
 *
 * Argument MUST be a literal integer.
 *
 * @api
 */
#[Attribute(Attribute::TARGET_PARAMETER | Attribute::IS_REPEATABLE)]
final class RequiredUntil
{
    /**
     * @no-named-arguments
     */
    public function __construct(
        public int $version,
    ) {}
}
