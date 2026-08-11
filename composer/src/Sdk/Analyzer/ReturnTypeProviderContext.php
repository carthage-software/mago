<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;

/** @api */
final class ReturnTypeProviderContext
{
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly Invocation $invocation,
        public readonly TypeComparator $types,
        public readonly CancellationTokenInterface $cancellation,
    ) {}
}
