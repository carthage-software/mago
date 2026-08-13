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
    /**
     * @var list<InitializationHook>
     */
    private array $initializationHooks = [];

    /**
     * @var list<FunctionReturnTypeProvider>
     */
    private array $functionReturnTypeProviders = [];

    /**
     * @var list<MethodReturnTypeProvider>
     */
    private array $methodReturnTypeProviders = [];

    /**
     * @var list<BeforeAnalysisHook>
     */
    private array $beforeAnalysisHooks = [];

    /**
     * @var list<AfterFileAnalysisHook>
     */
    private array $afterFileAnalysisHooks = [];

    /**
     * @var list<AfterAnalysisHook>
     */
    private array $afterAnalysisHooks = [];

    public function registerInitializationHook(InitializationHook $hook): void
    {
        $this->initializationHooks[] = $hook;
    }

    public function registerFunctionReturnTypeProvider(FunctionReturnTypeProvider $provider): void
    {
        $this->functionReturnTypeProviders[] = $provider;
    }

    public function registerMethodReturnTypeProvider(MethodReturnTypeProvider $provider): void
    {
        $this->methodReturnTypeProviders[] = $provider;
    }

    public function registerBeforeAnalysisHook(BeforeAnalysisHook $hook): void
    {
        $this->beforeAnalysisHooks[] = $hook;
    }

    public function registerAfterFileAnalysisHook(AfterFileAnalysisHook $hook): void
    {
        $this->afterFileAnalysisHooks[] = $hook;
    }

    public function registerAfterAnalysisHook(AfterAnalysisHook $hook): void
    {
        $this->afterAnalysisHooks[] = $hook;
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

    /**
     * @internal
     *
     * @return list<InitializationHook>
     */
    public function getInitializationHooks(): array
    {
        return $this->initializationHooks;
    }

    /**
     * @internal
     *
     * @return list<BeforeAnalysisHook>
     */
    public function getBeforeAnalysisHooks(): array
    {
        return $this->beforeAnalysisHooks;
    }

    /**
     * @internal
     *
     * @return list<AfterFileAnalysisHook>
     */
    public function getAfterFileAnalysisHooks(): array
    {
        return $this->afterFileAnalysisHooks;
    }

    /**
     * @internal
     *
     * @return list<AfterAnalysisHook>
     */
    public function getAfterAnalysisHooks(): array
    {
        return $this->afterAnalysisHooks;
    }
}
