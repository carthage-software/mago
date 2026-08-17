<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\AfterAnalysisHook;
use Mago\Sdk\Analyzer\AfterFileAnalysisHook;
use Mago\Sdk\Analyzer\AttributedEntryPoint;
use Mago\Sdk\Analyzer\BeforeAnalysisHook;
use Mago\Sdk\Analyzer\ClassInitializerProvider;
use Mago\Sdk\Analyzer\ClassLikeAnalysisHook;
use Mago\Sdk\Analyzer\ClassLikeTarget;
use Mago\Sdk\Analyzer\ClassTarget;
use Mago\Sdk\Analyzer\FunctionAssertionProvider;
use Mago\Sdk\Analyzer\FunctionReturnTypeProvider;
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\InitializationHook;
use Mago\Sdk\Analyzer\IssueFilterHook;
use Mago\Sdk\Analyzer\MethodAssertionProvider;
use Mago\Sdk\Analyzer\MethodCallAnalysisHook;
use Mago\Sdk\Analyzer\MethodReturnTypeProvider;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\NodeAnalysisHook;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PropertyInitializationProvider;
use Mago\Sdk\Analyzer\PropertyTarget;
use Mago\Sdk\Analyzer\PropertyTypeProvider;
use Mago\Sdk\Syntax\NodeKind;

/**
 * @internal
 * @mago-expect lint:excessive-parameter-list
 */
final class RegisteredPlugin
{
    /**
     * @param non-empty-string $extension
     * @param list<RegisteredTargetedCallback<FunctionReturnTypeProvider, FunctionTarget>> $functionProviders
     * @param list<RegisteredTargetedCallback<MethodReturnTypeProvider, MethodTarget>> $methodProviders
     * @param list<RegisteredTargetedCallback<FunctionAssertionProvider, FunctionTarget>> $functionAssertionProviders
     * @param list<RegisteredTargetedCallback<MethodAssertionProvider, MethodTarget>> $methodAssertionProviders
     * @param list<RegisteredTargetedCallback<PropertyTypeProvider, PropertyTarget>> $propertyProviders
     * @param list<RegisteredTargetedCallback<PropertyInitializationProvider, PropertyTarget>> $propertyInitializationProviders
     * @param list<RegisteredTargetedCallback<ClassInitializerProvider, ClassTarget>> $classInitializerProviders
     * @param list<MethodTarget> $entryPoints
     * @param list<AttributedEntryPoint> $attributedEntryPoints
     * @param list<RegisteredTargetedCallback<IssueFilterHook, string>> $issueFilterHooks
     * @param list<InitializationHook> $initializationHooks
     * @param list<BeforeAnalysisHook> $beforeAnalysisHooks
     * @param list<AfterFileAnalysisHook> $afterFileAnalysisHooks
     * @param list<RegisteredTargetedCallback<NodeAnalysisHook, NodeKind>> $nodeAnalysisHooks
     * @param array<string, non-empty-list<RegisteredTargetedCallback<NodeAnalysisHook, NodeKind>>> $nodeAnalysisHooksByNodeKind
     * @param list<RegisteredTargetedCallback<MethodCallAnalysisHook, MethodTarget>> $methodCallAnalysisHooks
     * @param array<int<0, 65535>, RegisteredTargetedCallback<MethodCallAnalysisHook, MethodTarget>> $methodCallAnalysisHooksByIndex
     * @param list<RegisteredTargetedCallback<ClassLikeAnalysisHook, ClassLikeTarget>> $classLikeAnalysisHooks
     * @param array<int<0, 65535>, RegisteredTargetedCallback<ClassLikeAnalysisHook, ClassLikeTarget>> $classLikeAnalysisHooksByIndex
     * @param list<AfterAnalysisHook> $afterAnalysisHooks
     */
    public function __construct(
        public readonly int $index,
        public readonly string $extension,
        public readonly Plugin $plugin,
        public readonly PluginDefinition $definition,
        public readonly array $functionProviders,
        public readonly array $methodProviders,
        public readonly array $functionAssertionProviders,
        public readonly array $methodAssertionProviders,
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
        public readonly array $classLikeAnalysisHooks,
        public readonly array $classLikeAnalysisHooksByIndex,
        public readonly array $afterAnalysisHooks,
        public readonly bool $memoizeProviders,
    ) {}
}
