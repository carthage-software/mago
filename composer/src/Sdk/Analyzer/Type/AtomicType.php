<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Type;

/**
 * @api
 */
interface AtomicType
{
    public function __toString(): string;
}
