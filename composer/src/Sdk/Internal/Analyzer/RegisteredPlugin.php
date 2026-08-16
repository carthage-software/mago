<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\AfterAnalysisHook;
use Mago\Sdk\Analyzer\AfterFileAnalysisHook;
use Mago\Sdk\Analyzer\AttributedEntryPoint;
use Mago\Sdk\Analyzer\BeforeAnalysisHook;
use Mago\Sdk\Analyzer\InitializationHook;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;

/**
 * @internal
 * @mago-expect lint:excessive-parameter-list
 */
final class RegisteredPlugin
{
    /**
     * @param non-empty-string $extension
     * @param list<RegisteredFunctionReturnTypeProvider> $functionProviders
     * @param list<RegisteredMethodReturnTypeProvider> $methodProviders
     * @param list<RegisteredPropertyTypeProvider> $propertyProviders
     * @param list<RegisteredPropertyInitializationProvider> $propertyInitializationProviders
     * @param list<RegisteredClassInitializerProvider> $classInitializerProviders
     * @param list<MethodTarget> $entryPoints
     * @param list<AttributedEntryPoint> $attributedEntryPoints
     * @param list<RegisteredIssueFilterHook> $issueFilterHooks
     * @param list<InitializationHook> $initializationHooks
     * @param list<BeforeAnalysisHook> $beforeAnalysisHooks
     * @param list<AfterFileAnalysisHook> $afterFileAnalysisHooks
     * @param list<RegisteredNodeAnalysisHook> $nodeAnalysisHooks
     * @param array<string, non-empty-list<RegisteredNodeAnalysisHook>> $nodeAnalysisHooksByNodeKind
     * @param list<RegisteredMethodCallAnalysisHook> $methodCallAnalysisHooks
     * @param array<int<0, 65535>, RegisteredMethodCallAnalysisHook> $methodCallAnalysisHooksByIndex
     * @param list<AfterAnalysisHook> $afterAnalysisHooks
     */
    public function __construct(
        public readonly int $index,
        public readonly string $extension,
        public readonly Plugin $plugin,
        public readonly PluginDefinition $definition,
        public readonly array $functionProviders,
        public readonly array $methodProviders,
        public readonly array $propertyProviders,
        public readonly array $propertyInitializationProviders,
        public readonly array $classInitializerProviders,
        public readonly array $entryPoints,
        public readonly array $attributedEntryPoints,
        public readonly array $issueFilterHooks,
        public readonly array $initializationHooks,
        public readonly array $beforeAnalysisHooks,
        public readonly array $afterFileAnalysisHooks,
        public readonly array $nodeAnalysisHooks,
        public readonly array $nodeAnalysisHooksByNodeKind,
        public readonly array $methodCallAnalysisHooks,
        public readonly array $methodCallAnalysisHooksByIndex,
        public readonly array $afterAnalysisHooks,
        public readonly bool $memoizeProviders,
    ) {}
}
