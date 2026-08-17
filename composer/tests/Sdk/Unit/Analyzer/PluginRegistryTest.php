<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\ClassLikeAnalysisHook;
use Mago\Sdk\Analyzer\ClassLikeTarget;
use Mago\Sdk\Analyzer\ClassTarget;
use Mago\Sdk\Analyzer\CodebaseScanContext;
use Mago\Sdk\Analyzer\CodebaseScanHook;
use Mago\Sdk\Analyzer\MethodCallAnalysisHook;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\NodeAnalysisContext;
use Mago\Sdk\Analyzer\NodeAnalysisHook;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Syntax\NodeKind;
use PHPUnit\Framework\TestCase;

final class PluginRegistryTest extends TestCase
{
    public function testProviderMemoizationIsExplicitlyEnabled(): void
    {
        $registry = new PluginRegistry();

        self::assertFalse($registry->shouldMemoizeProviders());

        $registry->enableProviderMemoization();

        self::assertTrue($registry->shouldMemoizeProviders());
    }

    public function testNodeAnalysisHooksPreserveRegistrationOrder(): void
    {
        $registry = new PluginRegistry();
        $first = self::nodeAnalysisHook(NodeKind::FunctionCall);
        $second = self::nodeAnalysisHook(NodeKind::MethodCall);

        $registry->registerNodeAnalysisHook($first);
        $registry->registerNodeAnalysisHook($second);

        self::assertSame([$first, $second], $registry->getNodeAnalysisHooks());
    }

    public function testCodebaseScanHooksPreserveRegistrationOrder(): void
    {
        $registry = new PluginRegistry();
        $first = self::codebaseScanHook('database/migrations/**/*.php');
        $second = self::codebaseScanHook('config/*.php');

        $registry->registerCodebaseScanHook($first);
        $registry->registerCodebaseScanHook($second);

        self::assertSame([$first, $second], $registry->getCodebaseScanHooks());
    }

    public function testMethodCallAnalysisHooksPreserveRegistrationOrder(): void
    {
        $registry = new PluginRegistry();
        $first = self::methodCallAnalysisHook(MethodTarget::exact('FrameworkTestCase', 'assert*'));
        $second = self::methodCallAnalysisHook(MethodTarget::exact('MockObject', 'method'));

        $registry->registerMethodCallAnalysisHook($first);
        $registry->registerMethodCallAnalysisHook($second);

        self::assertSame([$first, $second], $registry->getMethodCallAnalysisHooks());
    }

    public function testClassLikeAnalysisHooksPreserveRegistrationOrder(): void
    {
        $registry = new PluginRegistry();
        $first = self::classLikeAnalysisHook(ClassLikeTarget::descendantsOf('FrameworkTestCase'));
        $second = self::classLikeAnalysisHook(ClassLikeTarget::descendantsOf('FrameworkExtension'));

        $registry->registerClassLikeAnalysisHook($first);
        $registry->registerClassLikeAnalysisHook($second);

        self::assertSame([$first, $second], $registry->getClassLikeAnalysisHooks());
    }

    public function testFrameworkEntryPointsPreserveRegistrationOrder(): void
    {
        $registry = new PluginRegistry();
        $first = MethodTarget::exact('FrameworkTestCase', 'test*');
        $second = MethodTarget::exact('FrameworkTestCase', 'setUp');

        $registry->registerEntryPoint($first);
        $registry->registerEntryPoint($second);
        $registry->registerAttributedEntryPoint('FrameworkTestCase', 'FrameworkTest');
        $registry->registerAttributedEntryPoint(ClassTarget::exact('FrameworkTestCase'), 'FrameworkDataProvider');

        self::assertSame([$first, $second], $registry->getEntryPoints());

        $attributed = $registry->getAttributedEntryPoints();
        self::assertCount(2, $attributed);
        self::assertSame('FrameworkTestCase', $attributed[0]->class->class);
        self::assertSame('FrameworkTest', $attributed[0]->attribute);
        self::assertSame('FrameworkTestCase', $attributed[1]->class->class);
        self::assertSame('FrameworkDataProvider', $attributed[1]->attribute);
    }

    private static function nodeAnalysisHook(NodeKind $target): NodeAnalysisHook
    {
        return new class($target) implements NodeAnalysisHook {
            public function __construct(
                private readonly NodeKind $target,
            ) {}

            public function getTargets(): array
            {
                return [$this->target];
            }

            public function getRequirements(): array
            {
                return [];
            }

            public function analyze(NodeAnalysisContext $context): void {}
        };
    }

    private static function codebaseScanHook(string $pattern): CodebaseScanHook
    {
        return new class($pattern) implements CodebaseScanHook {
            public function __construct(
                private readonly string $pattern,
            ) {}

            public function getTargets(): array
            {
                return [$this->pattern];
            }

            public function scan(CodebaseScanContext $context): void {}
        };
    }

    private static function methodCallAnalysisHook(MethodTarget $target): MethodCallAnalysisHook
    {
        return new class($target) implements MethodCallAnalysisHook {
            public function __construct(
                private readonly MethodTarget $target,
            ) {}

            public function getTargets(): array
            {
                return [$this->target];
            }

            public function getRequirements(): array
            {
                return [];
            }

            public function analyze(NodeAnalysisContext $context): void {}
        };
    }

    private static function classLikeAnalysisHook(ClassLikeTarget $target): ClassLikeAnalysisHook
    {
        return new class($target) implements ClassLikeAnalysisHook {
            public function __construct(
                private readonly ClassLikeTarget $target,
            ) {}

            public function getTargets(): array
            {
                return [$this->target];
            }

            public function getRequirements(): array
            {
                return [];
            }

            public function analyze(NodeAnalysisContext $context): void {}
        };
    }
}
