<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;

/** @internal */
final class RegisteredPlugin
{
    /**
     * @param non-empty-string $extension
     * @param list<RegisteredFunctionReturnTypeProvider> $functionProviders
     * @param list<RegisteredMethodReturnTypeProvider> $methodProviders
     */
    public function __construct(
        public readonly string $extension,
        public readonly Plugin $plugin,
        public readonly PluginDefinition $definition,
        public readonly array $functionProviders,
        public readonly array $methodProviders,
    ) {}
}
