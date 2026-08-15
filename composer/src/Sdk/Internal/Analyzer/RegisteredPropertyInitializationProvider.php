<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\PropertyInitializationProvider;
use Mago\Sdk\Analyzer\PropertyTarget;

/** @internal */
final class RegisteredPropertyInitializationProvider
{
    /**
     * @param int<0, 65535> $index
     * @param non-empty-string $plugin
     * @param non-empty-list<PropertyTarget> $targets
     */
    public function __construct(
        public readonly int $index,
        public readonly string $plugin,
        public readonly PropertyInitializationProvider $provider,
        public readonly array $targets,
    ) {}
}
