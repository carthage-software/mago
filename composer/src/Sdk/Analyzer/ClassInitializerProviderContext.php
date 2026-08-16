<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Analyzer\Metadata\ClassLikeMetadata;
use Mago\Sdk\CancellationTokenInterface;
use Mago\Sdk\PHPVersion;

/**
 * Context for identifying framework lifecycle methods on one class-like.
 *
 * @api
 */
final class ClassInitializerProviderContext
{
    public function __construct(
        public readonly PHPVersion $phpVersion,
        public readonly Codebase $codebase,
        public readonly ClassLikeMetadata $class,
        public readonly CancellationTokenInterface $cancellation,
    ) {}
}
