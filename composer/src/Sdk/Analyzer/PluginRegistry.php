<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Collects the semantic providers contributed by one analyzer plugin.
 *
 * @api
 */
final class PluginRegistry
{
    /** @var list<FunctionReturnTypeProvider> */
    private array $functionReturnTypeProviders = [];

    /** @var list<MethodReturnTypeProvider> */
    private array $methodReturnTypeProviders = [];

    public function registerFunctionReturnTypeProvider(FunctionReturnTypeProvider $provider): void
    {
        $this->functionReturnTypeProviders[] = $provider;
    }

    public function registerMethodReturnTypeProvider(MethodReturnTypeProvider $provider): void
    {
        $this->methodReturnTypeProviders[] = $provider;
    }

    /**
     * @internal
     * @return list<FunctionReturnTypeProvider>
     */
    public function getFunctionReturnTypeProviders(): array
    {
        return $this->functionReturnTypeProviders;
    }

    /**
     * @internal
     * @return list<MethodReturnTypeProvider>
     */
    public function getMethodReturnTypeProviders(): array
    {
        return $this->methodReturnTypeProviders;
    }
}
