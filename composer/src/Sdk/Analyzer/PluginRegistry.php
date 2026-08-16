<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

/**
 * Collects the semantic providers contributed by one analyzer plugin.
 *
 * @api
 * @mago-expect lint:too-many-methods
 * @mago-expect lint:too-many-properties
 */
final class PluginRegistry
{
    private bool $memoizeProviders = false;

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
     * @var list<PropertyTypeProvider>
     */
    private array $propertyTypeProviders = [];

    /**
     * @var list<PropertyInitializationProvider>
     */
    private array $propertyInitializationProviders = [];

    /**
     * @var list<IssueFilterHook>
     */
    private array $issueFilterHooks = [];

    /**
     * @var list<BeforeAnalysisHook>
     */
    private array $beforeAnalysisHooks = [];

    /**
     * @var list<AfterFileAnalysisHook>
     */
    private array $afterFileAnalysisHooks = [];

    /**
     * @var list<NodeAnalysisHook>
     */
    private array $nodeAnalysisHooks = [];

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

    public function registerPropertyTypeProvider(PropertyTypeProvider $provider): void
    {
        $this->propertyTypeProviders[] = $provider;
    }

    public function registerPropertyInitializationProvider(PropertyInitializationProvider $provider): void
    {
        $this->propertyInitializationProviders[] = $provider;
    }

    public function registerIssueFilterHook(IssueFilterHook $hook): void
    {
        $this->issueFilterHooks[] = $hook;
    }

    public function registerBeforeAnalysisHook(BeforeAnalysisHook $hook): void
    {
        $this->beforeAnalysisHooks[] = $hook;
    }

    public function registerAfterFileAnalysisHook(AfterFileAnalysisHook $hook): void
    {
        $this->afterFileAnalysisHooks[] = $hook;
    }

    public function registerNodeAnalysisHook(NodeAnalysisHook $hook): void
    {
        $this->nodeAnalysisHooks[] = $hook;
    }

    public function registerAfterAnalysisHook(AfterAnalysisHook $hook): void
    {
        $this->afterAnalysisHooks[] = $hook;
    }

    /**
     * Memoize function and method provider results for identical invocations within one frozen analysis generation.
     *
     * This includes callable-signature results. Enable it only when every registered function and method provider is
     * deterministic and does not depend on source locations, invocation order, or externally mutable state.
     */
    public function enableProviderMemoization(): void
    {
        $this->memoizeProviders = true;
    }

    /** @internal */
    public function shouldMemoizeProviders(): bool
    {
        return $this->memoizeProviders;
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
     * @return list<PropertyTypeProvider>
     */
    public function getPropertyTypeProviders(): array
    {
        return $this->propertyTypeProviders;
    }

    /**
     * @internal
     * @return list<PropertyInitializationProvider>
     */
    public function getPropertyInitializationProviders(): array
    {
        return $this->propertyInitializationProviders;
    }

    /**
     * @internal
     *
     * @return list<IssueFilterHook>
     */
    public function getIssueFilterHooks(): array
    {
        return $this->issueFilterHooks;
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
     * @return list<NodeAnalysisHook>
     */
    public function getNodeAnalysisHooks(): array
    {
        return $this->nodeAnalysisHooks;
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
