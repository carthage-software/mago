<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;

/**
 * @api
 */
final class PropertyTypeProviderContext
{
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly Codebase $codebase,
        public readonly PropertyAccess $access,
        public readonly TypeComparator $types,
        public readonly CancellationTokenInterface $cancellation,
    ) {}
}
