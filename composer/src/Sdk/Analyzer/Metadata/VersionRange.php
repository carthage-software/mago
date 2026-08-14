<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer\Metadata;

use Mago\Sdk\PHPVersion;

/**
 * @api
 */
final class VersionRange
{
    public function __construct(
        public readonly ?PHPVersion $minimum,
        public readonly ?PHPVersion $maximum,
    ) {}
}
