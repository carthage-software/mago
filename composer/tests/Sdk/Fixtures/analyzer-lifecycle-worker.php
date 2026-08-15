<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\AfterAnalysisContext;
use Mago\Sdk\Analyzer\AfterAnalysisHook;
use Mago\Sdk\Analyzer\AfterFileAnalysisContext;
use Mago\Sdk\Analyzer\AfterFileAnalysisHook;
use Mago\Sdk\Analyzer\BeforeAnalysisContext;
use Mago\Sdk\Analyzer\BeforeAnalysisHook;
use Mago\Sdk\Analyzer\InitializationContext;
use Mago\Sdk\Analyzer\InitializationHook;
use Mago\Sdk\Analyzer\LifecycleContext;
use Mago\Sdk\Analyzer\Metadata\ClassLikeMetadata;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\ReferenceKind;
use Mago\Sdk\Analyzer\ReferenceOrigin;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Extension;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Reporting\Safety;
use Mago\Sdk\Reporting\TextEdit;
use Mago\Sdk\Span;
use Mago\Sdk\Syntax\NodeKind;
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
use function str_contains;
use function strlen;
use function usleep;

use const FILE_APPEND;
use const JSON_THROW_ON_ERROR;
use const LOCK_EX;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

/**
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 */
final class LifecycleProofHook implements
    InitializationHook,
    BeforeAnalysisHook,
    AfterFileAnalysisHook,
    AfterAnalysisHook
{
    public function __construct(
        private readonly string $plugin,
        private readonly string $auditLog,
    ) {}

    public function initialize(InitializationContext $context): void
    {
        if ($this->plugin === 'lifecycle-one') {
            $context->addStub('lifecycle.php', <<<'PHP'
                <?php

                declare(strict_types=1);

                /**
                 * @template-covariant T of int = int
                 * @type Answer = int
                 * @mixin stdClass
                 * @property string $magic
                 * @seal-methods
                 * @inheritors LifecycleClass0
                 */
                class ExtensionProvided
                {
                    public int $value = 42;
                    public const int ANSWER = 42;

                    public function answer(): int
                    {
                        return 42;
                    }
                }

                /** @template T */
                function extension_answer(int $fallback = 0): int
                {
                    return $fallback;
                }

                const EXTENSION_ANSWER = 42;
                PHP);
        }

        $this->record('initialize', null);
    }

    public function beforeAnalysis(BeforeAnalysisContext $context): void
    {
        $base = $this->verifySharedContext($context);
        if ($this->plugin === 'lifecycle-one' && $context->codebase->getConstant('ENABLE_FRAMEWORK_ACTION') !== null) {
            $frameworkAction = new MemberIdentifier('LifecycleClass0', 'frameworkAction');
            $context->references->add('Symfony\Kernel', $frameworkAction);
            $context->references->add(ReferenceOrigin::file('config/routes.php'), $frameworkAction);
        }
        $this->record('before', null);
        $context->report(Level::Help, 'before', Issue::at('Before-analysis hook ran.', $base->location));
    }

    public function afterFileAnalysis(AfterFileAnalysisContext $context): void
    {
        $this->verifySharedContext($context);
        $analysis = $context->analysis;
        $source = $analysis->getSourceFile();
        if (
            $source !== $analysis->getSourceFile()
            || $source->path !== $analysis->file
            || strlen($source->contents) !== $analysis->size
            || !str_contains($source->contents, '/* exact-in-memory-source:')
        ) {
            throw new RuntimeException('The exact analyzed in-memory source did not round-trip lazily.');
        }

        $programs = $source->getNodes(NodeKind::Program);
        if (count($programs) !== 1 || $programs[0]->parentId !== null || $source->getChildren($programs[0]) === []) {
            throw new RuntimeException('The complete analyzed syntax tree did not round-trip.');
        }

        $resolvedNames = $source->getResolvedNames();
        if ($resolvedNames === [] || $source->getResolvedName($resolvedNames[0]->span) !== $resolvedNames[0]) {
            throw new RuntimeException('The analyzed source lost its resolved names.');
        }

        $expressionTypes = $analysis->getAllExpressionTypes();
        if (count($expressionTypes) !== $analysis->expressionCount) {
            throw new RuntimeException('The complete expression-type result has the wrong size.');
        }

        if ($expressionTypes !== []) {
            $first = $expressionTypes[0];
            $expressionNode = null;
            foreach ($source->getNodes() as $node) {
                if ($node->span->start !== $first->span->start || $node->span->end !== $first->span->end) {
                    continue;
                }

                $expressionNode = $node;
                break;
            }

            if ($expressionNode === null) {
                throw new RuntimeException('An analyzed expression has no matching syntax node.');
            }

            $types = $analysis->getMultipleExpressionTypes([
                $expressionNode,
                new Span($analysis->size, $analysis->size),
            ]);
            if ($types[0] === null || !$context->types->equals($types[0], $first->type)) {
                throw new RuntimeException('A syntax-node expression type did not round-trip.');
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
            Issue::new('After-file hook ran.', new Span(0, $analysis->size))->withEdit(TextEdit::replace(
                new Span(0, min(5, $analysis->size)),
                '<?php',
            )->withSafety(Safety::PotentiallyUnsafe)),
        );
    }

    /** @mago-expect lint:halstead */
    public function afterAnalysis(AfterAnalysisContext $context): void
    {
        $base = $this->verifySharedContext($context);
        $child = $context->codebase->getClass('LifecycleClass1');
        $external = $context->codebase->getClass('ExtensionProvided');
        if ($child === null || $external === null) {
            throw new RuntimeException('The final hook cannot query the child class.');
        }

        $frameworkReferenceEnabled = $context->codebase->getConstant('ENABLE_FRAMEWORK_ACTION') !== null;
        $project = $context->analysis;
        $expectedIssueCount = ($frameworkReferenceEnabled ? 3 : 4) + (count($project->files) * 2) - 1;
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

        $frameworkAction = new MemberIdentifier('LifecycleClass0', 'frameworkAction');
        $referencesToAction = $project->references->getReferencesTo($frameworkAction);
        [$referencesFromKernel, $referencesFromConsumer] = $project->references->getMultipleReferencesFrom([
            'Symfony\Kernel',
            'extension_consumer',
        ]);
        $kernelReferencesAction = false;
        $routesReferenceAction = false;
        foreach ($referencesToAction as $reference) {
            if ($reference->source->file !== 'config/routes.php') {
                continue;
            }

            $routesReferenceAction = true;
            break;
        }
        foreach ($referencesFromKernel as $reference) {
            if (!$reference->target instanceof MemberIdentifier) {
                continue;
            }

            if ($reference->target->class !== 'lifecycleclass0' || $reference->target->member !== 'frameworkaction') {
                continue;
            }

            $kernelReferencesAction = true;
            break;
        }

        $consumerReferencesAnswer = false;
        foreach ($referencesFromConsumer as $reference) {
            if (!$reference->target instanceof MemberIdentifier) {
                continue;
            }

            if ($reference->target->class !== 'extensionprovided' || $reference->target->member !== 'answer') {
                continue;
            }

            $consumerReferencesAnswer = true;
            break;
        }
        if (
            count($referencesToAction) !== ($frameworkReferenceEnabled ? 2 : 0)
            || count($referencesFromKernel) !== ($frameworkReferenceEnabled ? 2 : 0)
            || $frameworkReferenceEnabled && $referencesToAction[0]->kind !== ReferenceKind::Body
            || $frameworkReferenceEnabled && $referencesToAction[0]->source->symbol !== 'symfony\kernel'
            || $kernelReferencesAction !== $frameworkReferenceEnabled
            || $routesReferenceAction !== $frameworkReferenceEnabled
            || !$consumerReferencesAnswer
        ) {
            throw new RuntimeException('The final merged native and synthetic references did not round-trip.');
        }

        [$knownReferences, $missingReferences] = $project->references->getMultipleReferencesTo([
            $frameworkAction,
            'DefinitelyMissing',
        ]);
        if (count($knownReferences) !== ($frameworkReferenceEnabled ? 2 : 0) || $missingReferences !== []) {
            throw new RuntimeException('A batched final symbol-reference query returned the wrong graph edges.');
        }

        $this->record('after', null);
        $context->report(
            Level::Help,
            'after',
            Issue::at('After-analysis hook ran.', $base->location)->withSecondaryLocation(
                $external->location,
                'External-stub lifecycle annotation.',
            )->withEdit(TextEdit::replaceAt($base->location, '')->withSafety(Safety::Unsafe)),
        );
    }

    private function verifySharedContext(LifecycleContext $context): ClassLikeMetadata
    {
        [$base, $missing] = $context->codebase->getMultipleClasses(['LifecycleClass0', 'DefinitelyMissing']);
        $extensionClass = $context->codebase->getClass('ExtensionProvided');
        $extensionFunction = $context->codebase->getFunction('extension_answer');
        if ($base === null || $missing !== null || !$context->codebase->classExists('LifecycleClass0')) {
            throw new RuntimeException('A lifecycle hook cannot query host classes.');
        }

        if (!$context->types->isContainedBy(Type::literalInt(1), Type::int())) {
            throw new RuntimeException('A lifecycle hook cannot compare types.');
        }

        if ($extensionClass === null) {
            throw new RuntimeException('A lifecycle hook cannot query an external-stub class.');
        }

        if (
            count($extensionClass->templates) !== 1
            || count($extensionClass->typeAliases) !== 1
            || count($extensionClass->mixins) !== 1
            || $extensionClass->magicProperties !== ['$magic']
            || $extensionClass->sealedMethods !== true
            || $extensionClass->permittedInheritors !== ['lifecycleclass0']
        ) {
            throw new RuntimeException('An external-stub class lost rich metadata.');
        }

        if (
            $context->codebase->getMethod('ExtensionProvided', 'answer') === null
            || $context->codebase->getProperty('ExtensionProvided', '$value') === null
            || $context->codebase->getClassConstant('ExtensionProvided', 'ANSWER') === null
        ) {
            throw new RuntimeException('An external-stub class lost member metadata.');
        }

        if ($extensionFunction === null || count($extensionFunction->templates) !== 1) {
            throw new RuntimeException('A lifecycle hook cannot query an external-stub function.');
        }

        if ($context->codebase->getConstant('EXTENSION_ANSWER') === null) {
            throw new RuntimeException('A lifecycle hook cannot query an external-stub constant.');
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
        $registry->registerInitializationHook($hook);
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
