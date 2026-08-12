<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\AfterAnalysisContext;
use Mago\Sdk\Analyzer\AfterAnalysisHook;
use Mago\Sdk\Analyzer\AfterFileAnalysisContext;
use Mago\Sdk\Analyzer\AfterFileAnalysisHook;
use Mago\Sdk\Analyzer\BeforeAnalysisContext;
use Mago\Sdk\Analyzer\BeforeAnalysisHook;
use Mago\Sdk\Analyzer\Definition\ClassConstantDefinition;
use Mago\Sdk\Analyzer\Definition\ClassLikeDefinition;
use Mago\Sdk\Analyzer\Definition\ConstantDefinition;
use Mago\Sdk\Analyzer\Definition\FunctionDefinition;
use Mago\Sdk\Analyzer\Definition\MethodDefinition;
use Mago\Sdk\Analyzer\Definition\ParameterDefinition;
use Mago\Sdk\Analyzer\Definition\PropertyDefinition;
use Mago\Sdk\Analyzer\Definition\TemplateDefinition;
use Mago\Sdk\Analyzer\LifecycleContext;
use Mago\Sdk\Analyzer\Metadata\ClassLikeMetadata;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\Variance;
use Mago\Sdk\Extension;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Span;
use Mago\Sdk\Worker;
use RuntimeException;

use function count;
use function dirname;
use function file_put_contents;
use function getenv;
use function getmypid;
use function is_string;
use function json_encode;
use function min;
use function usleep;

use const FILE_APPEND;
use const JSON_THROW_ON_ERROR;
use const LOCK_EX;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

/**
 * @mago-expect lint:cyclomatic-complexity
 */
final class LifecycleProofHook implements BeforeAnalysisHook, AfterFileAnalysisHook, AfterAnalysisHook
{
    public function __construct(
        private readonly string $plugin,
        private readonly string $auditLog,
    ) {}

    public function beforeAnalysis(BeforeAnalysisContext $context): void
    {
        if ($this->plugin === 'lifecycle-one') {
            $class = $context->codebase->removeClassLike('ExtensionProvided') ?? new ClassLikeDefinition(
                name: 'ExtensionProvided',
                methods: [new MethodDefinition('answer', returnType: Type::int())],
                properties: [new PropertyDefinition('$value', Type::int())],
                magicProperties: [new PropertyDefinition('$magic', Type::string())],
                constants: [new ClassConstantDefinition('ANSWER', Type::int(), valueType: Type::literalInt(42))],
                templates: [new TemplateDefinition('T', Type::mixed(), Type::int(), Variance::Covariant, true)],
                typeAliases: ['Answer' => Type::int()],
                mixins: [Type::namedObject('stdClass')],
                sealedMethods: true,
                permittedInheritors: ['LifecycleClass0'],
            );
            $function = $context->codebase->removeFunction('extension_answer') ?? new FunctionDefinition(
                'extension_answer',
                [new ParameterDefinition('$fallback', Type::int(), defaultType: Type::literalInt(0))],
                Type::int(),
                templates: [new TemplateDefinition('T', Type::mixed())],
            );
            $constant = $context->codebase->removeConstant('EXTENSION_ANSWER') ?? new ConstantDefinition(
                'EXTENSION_ANSWER',
                Type::int(),
                Type::literalInt(42),
            );
            $context->codebase->insertClassLike($class);
            $context->codebase->insertFunction($function);
            $context->codebase->insertConstant($constant);
        }

        $base = $this->verifySharedContext($context);
        $this->record('before', null);
        $context->report(Level::Help, 'before', Issue::at('Before-analysis hook ran.', $base->location));
    }

    public function afterFileAnalysis(AfterFileAnalysisContext $context): void
    {
        $this->verifySharedContext($context);
        $analysis = $context->analysis;
        $expressionTypes = $analysis->getAllExpressionTypes();
        if (count($expressionTypes) !== $analysis->expressionCount) {
            throw new RuntimeException('The complete expression-type result has the wrong size.');
        }

        if ($expressionTypes !== []) {
            $first = $expressionTypes[0];
            $types = $analysis->getMultipleExpressionTypes([$first->span, new Span($analysis->size, $analysis->size)]);
            if ($types[0] === null || !$context->types->equals($types[0], $first->type)) {
                throw new RuntimeException('A lazy expression type did not round-trip.');
            }
        }

        if (
            count($analysis->getInferredReturnTypes()) !== $analysis->inferredReturnCount
            || count($analysis->getInferredYieldKeyTypes()) !== $analysis->inferredYieldKeyCount
            || count($analysis->getInferredYieldValueTypes()) !== $analysis->inferredYieldValueCount
        ) {
            throw new RuntimeException('A lazy inferred-type result has the wrong size.');
        }

        $this->record('after-file', $analysis->file);
        usleep(2_000);
        $context->report(
            Level::Help,
            'after-file',
            Issue::new('After-file hook ran.', new Span(0, min(5, $analysis->size))),
        );
    }

    public function afterAnalysis(AfterAnalysisContext $context): void
    {
        $base = $this->verifySharedContext($context);
        $child = $context->codebase->getClass('LifecycleClass1');
        if ($child === null) {
            throw new RuntimeException('The final hook cannot query the child class.');
        }

        $project = $context->analysis;
        $expectedIssueCount = 2 + (count($project->files) * 2);
        if (count($project->files) !== 96 || $project->issueCount !== $expectedIssueCount) {
            throw new RuntimeException(
                "The final hook received {$project->issueCount} issues; expected {$expectedIssueCount}.",
            );
        }

        $names = [];
        foreach ($project->files as $file) {
            $names[] = $file->file;
            if (count($file->getAllExpressionTypes()) !== $file->expressionCount) {
                throw new RuntimeException('A retained file snapshot lost expression types.');
            }

            $file->getInferredReturnTypes();
            $file->getInferredYieldKeyTypes();
            $file->getInferredYieldValueTypes();
        }

        $files = $project->getMultipleFiles($names);
        foreach ($files as $index => $file) {
            if ($file !== $project->files[$index]) {
                throw new RuntimeException('Batched project file lookup lost object identity.');
            }
        }

        $this->record('after', null);
        $context->report(
            Level::Help,
            'after',
            Issue::at('After-analysis hook ran.', $base->location)->withSecondaryLocation(
                $child->location,
                'Cross-file lifecycle annotation.',
            ),
        );
    }

    private function verifySharedContext(LifecycleContext $context): ClassLikeMetadata
    {
        [$base, $missing] = $context->codebase->getMultipleClasses(['LifecycleClass0', 'DefinitelyMissing']);
        $extensionClass = $context->codebase->getClass('ExtensionProvided');
        $extensionFunction = $context->codebase->getFunction('extension_answer');
        if (
            $base === null
            || $missing !== null
            || !$context->codebase->classExists('LifecycleClass0')
            || !$context->types->isContainedBy(Type::literalInt(1), Type::int())
            || $extensionClass === null
            || count($extensionClass->templates) !== 1
            || count($extensionClass->typeAliases) !== 1
            || count($extensionClass->mixins) !== 1
            || $extensionClass->magicProperties !== ['$magic']
            || $extensionClass->sealedMethods !== true
            || $extensionClass->permittedInheritors !== ['lifecycleclass0']
            || $context->codebase->getMethod('ExtensionProvided', 'answer') === null
            || $context->codebase->getProperty('ExtensionProvided', '$value') === null
            || $context->codebase->getClassConstant('ExtensionProvided', 'ANSWER') === null
            || $extensionFunction === null
            || count($extensionFunction->templates) !== 1
            || $context->codebase->getConstant('EXTENSION_ANSWER') === null
        ) {
            throw new RuntimeException('A lifecycle hook cannot use its shared read-only services.');
        }

        return $base;
    }

    private function record(string $phase, ?string $file): void
    {
        $record = json_encode([$this->plugin, $phase, $file, getmypid()], JSON_THROW_ON_ERROR);
        if (file_put_contents($this->auditLog, $record . "\n", FILE_APPEND | LOCK_EX) === false) {
            throw new RuntimeException('Unable to append to the lifecycle audit log.');
        }
    }
}

/**
 * @mago-expect lint:single-class-per-file
 */
final class LifecycleProofPlugin implements Plugin
{
    public function __construct(
        private readonly string $identifier,
        private readonly string $auditLog,
    ) {}

    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition($this->identifier, $this->identifier, 'Exercises analyzer lifecycle hooks.');
    }

    public function register(PluginRegistry $registry): void
    {
        $hook = new LifecycleProofHook($this->identifier, $this->auditLog);
        $registry->registerBeforeAnalysisHook($hook);
        $registry->registerAfterFileAnalysisHook($hook);
        $registry->registerAfterAnalysisHook($hook);
    }
}

$auditLog = getenv('MAGO_LIFECYCLE_AUDIT_LOG');
if (!is_string($auditLog) || $auditLog === '') {
    throw new RuntimeException('MAGO_LIFECYCLE_AUDIT_LOG must name the lifecycle audit file.');
}

(new Worker(new Extension(
    identifier: 'mago/lifecycle-proof',
    name: 'Mago analyzer lifecycle proof',
    version: '1.0.0',
    analyzerPlugins: [
        new LifecycleProofPlugin('lifecycle-one', $auditLog),
        new LifecycleProofPlugin('lifecycle-two', $auditLog),
    ],
)))->run();
