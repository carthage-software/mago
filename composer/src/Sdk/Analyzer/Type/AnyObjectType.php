<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
final class AnyObjectType implements AtomicType
{
    public function __toString(): string
    {
        return 'object';
    }
}
