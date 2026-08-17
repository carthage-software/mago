<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\AfterAnalysisContext;
use Mago\Sdk\Analyzer\AfterAnalysisHook;
use Mago\Sdk\Analyzer\AfterFileAnalysisContext;
use Mago\Sdk\Analyzer\AfterFileAnalysisHook;
use Mago\Sdk\Analyzer\Assertion\TypeAssertion;
use Mago\Sdk\Analyzer\Assertion\TypeAssertionKind;
use Mago\Sdk\Analyzer\BeforeAnalysisContext;
use Mago\Sdk\Analyzer\BeforeAnalysisHook;
use Mago\Sdk\Analyzer\ClassLikeAnalysisHook;
use Mago\Sdk\Analyzer\ClassLikeTarget;
use Mago\Sdk\Analyzer\FileAnalysisRequirement;
use Mago\Sdk\Analyzer\InitializationContext;
use Mago\Sdk\Analyzer\InitializationHook;
use Mago\Sdk\Analyzer\LifecycleContext;
use Mago\Sdk\Analyzer\Metadata\ClassLikeMetadata;
use Mago\Sdk\Analyzer\Metadata\FunctionLikeKind as MetadataFunctionLikeKind;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\MethodCallAnalysisHook;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\NodeAnalysisContext;
use Mago\Sdk\Analyzer\NodeAnalysisHook;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\ReferenceKind;
use Mago\Sdk\Analyzer\ReferenceOrigin;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\CallableType;
use Mago\Sdk\Analyzer\Type\FunctionLikeIdentifier;
use Mago\Sdk\Analyzer\Type\FunctionLikeKind as IdentifierKind;
use Mago\Sdk\Analyzer\TypeComparison;
use Mago\Sdk\Extension;
use Mago\Sdk\Reporting\Issue;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Reporting\Safety;
use Mago\Sdk\Reporting\TextEdit;
use Mago\Sdk\Span;
use Mago\Sdk\Syntax\CallExpression;
use Mago\Sdk\Syntax\NodeKind;
use Mago\Sdk\Worker;
use RuntimeException;

use function array_values;
use function count;
use function dirname;
use function file_put_contents;
use function getenv;
use function getmypid;
use function in_array;
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
 * @mago-expect lint:too-many-methods
 */
final class LifecycleProofPlugin implements
    Plugin,
    InitializationHook,
    BeforeAnalysisHook,
    AfterFileAnalysisHook,
    NodeAnalysisHook,
    AfterAnalysisHook
{
    public function __construct(
        private readonly string $plugin,
        private readonly string $auditLog,
    ) {}

    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition($this->plugin, $this->plugin, 'Exercises analyzer lifecycle hooks.');
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerInitializationHook($this);
        $registry->registerBeforeAnalysisHook($this);
        $registry->registerAfterFileAnalysisHook($this);
        $registry->registerNodeAnalysisHook($this);
        $registry->registerMethodCallAnalysisHook(new LifecycleMethodCallHook($this->plugin, $this->auditLog));
        $registry->registerClassLikeAnalysisHook(new LifecycleClassLikeHook($this->plugin, $this->auditLog));
        $registry->registerAfterAnalysisHook($this);
    }

    public function getRequirements(): array
    {
        return [
            FileAnalysisRequirement::ExpressionTypes,
            FileAnalysisRequirement::TargetExpressionTypes,
            FileAnalysisRequirement::ReceiverType,
            FileAnalysisRequirement::ArgumentTypes,
            FileAnalysisRequirement::TargetSubtree,
            FileAnalysisRequirement::SourceText,
        ];
    }

    public function getTargets(): array
    {
        return [NodeKind::FunctionCall, NodeKind::MethodCall];
    }

    public function analyze(NodeAnalysisContext $context): void
    {
        if (
            $context->node->kind !== NodeKind::FunctionCall && $context->node->kind !== NodeKind::MethodCall
            || $context->source->path !== $context->analysis->file
            || $context->source->contents === ''
        ) {
            throw new RuntimeException('A targeted node hook received an inconsistent analysis snapshot.');
        }

        $call = CallExpression::fromNode($context->source, $context->node);
        if (
            $context->targetType === null
            || (string) $context->targetType !== (string) $context->analysis->getExpressionType($context->node)
            || count($context->argumentTypes) !== count($call->arguments)
            || $call->isFunction() && $context->receiverType !== null
            || $call->isMethod() && $context->receiverType === null
        ) {
            throw new RuntimeException('A targeted node hook received inconsistent fine-grained types.');
        }

        $this->record('node', $context->analysis->file);
    }

    public function initialize(InitializationContext $context): void
    {
        if ($this->plugin === 'lifecycle-one') {
            $context->addStub('lifecycle.php', <<<'PHP'
                <?php

                declare(strict_types=1);

                #[\Attribute(\Attribute::TARGET_METHOD | \Attribute::TARGET_PROPERTY)]
                final class ExtensionMarker
                {
                    public function __construct(public string $value, public bool $enabled = true) {}
                }

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
                    #[ExtensionMarker('property')]
                    public int $value = 42;
                    public const int ANSWER = 42;

                    #[ExtensionMarker(ExtensionProvided::class, enabled: false)]
                    public function answer(): int
                    {
                        return 42;
                    }

                    public function unrelated(): int
                    {
                        return 0;
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

    /** @mago-expect lint:halstead */
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

        if (
            $this->plugin === 'lifecycle-one'
            && $analysis->file === 'src/file5.php'
            && $context->codebase->getConstant('ENABLE_LATE_FRAMEWORK_ACTION') !== null
        ) {
            $context->references->add(
                new MemberIdentifier('LifecycleClass5', 'value'),
                new MemberIdentifier('LifecycleClass0', 'lateFrameworkAction'),
            );
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

        if ($analysis->file === 'src/file0.php') {
            $identifiers = [];
            foreach ($expressionTypes as $expressionType) {
                foreach ($expressionType->type->atomicTypes as $atomicType) {
                    if (!$atomicType instanceof CallableType) {
                        continue;
                    }

                    $identifier = $atomicType->signature?->source;
                    if ($identifier === null || $identifier->kind !== IdentifierKind::Closure) {
                        continue;
                    }

                    $identifiers[$identifier->name] = $identifier;
                }
            }

            $identifiers = array_values($identifiers);
            $functionLikes = $context->codebase->getMultipleFunctionLikes($identifiers);
            $kinds = [];
            foreach ($functionLikes as $index => $functionLike) {
                if ($functionLike === null) {
                    throw new RuntimeException('A closure identifier did not resolve to function-like metadata.');
                }

                $identifier = $identifiers[$index];
                if (!$functionLike->identifier->equals($identifier)) {
                    throw new RuntimeException('Function-like metadata did not preserve its stable identifier.');
                }
                if ($context->codebase->getFunctionLike($identifier) !== $functionLike) {
                    throw new RuntimeException('Function-like metadata did not preserve cache identity.');
                }
                $kinds[$functionLike->kind->name] = true;
            }

            if (
                !($kinds[MetadataFunctionLikeKind::Closure->name] ?? false)
                || !($kinds[MetadataFunctionLikeKind::ArrowFunction->name] ?? false)
            ) {
                throw new RuntimeException('Closure and arrow-function metadata did not round-trip by identifier.');
            }
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
        $incomplete = $context->codebase->getClass('IncompleteLifecycleClass');
        if ($child === null || $external === null || $incomplete === null) {
            throw new RuntimeException('The final hook cannot query project classes.');
        }

        if (
            $child->hasIncompleteHierarchy()
            || $child->unresolvedHierarchyDependencies !== []
            || !$incomplete->hasIncompleteHierarchy()
            || $incomplete->unresolvedHierarchyDependencies !== ['vendor\\unresolvable\\lifecyclecontract']
        ) {
            throw new RuntimeException('Class-like metadata lost hierarchy completeness information.');
        }

        $frameworkReferenceEnabled = $context->codebase->getConstant('ENABLE_FRAMEWORK_ACTION') !== null;
        $lateReferenceEnabled = $context->codebase->getConstant('ENABLE_LATE_FRAMEWORK_ACTION') !== null;
        $project = $context->analysis;
        $expectedIssueCount =
            3 + ($frameworkReferenceEnabled ? 0 : 1) + ($lateReferenceEnabled ? 0 : 1) + (count($project->files) * 2)
            - 1;
        if ($project->issueCount !== $expectedIssueCount) {
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
        $lateFrameworkAction = new MemberIdentifier('LifecycleClass0', 'lateFrameworkAction');
        $topLevelValue = new MemberIdentifier('LifecycleClass0', 'value');
        $topLevelProperty = new MemberIdentifier('LifecycleClass0', '$topLevelProperty');
        $closureSource = new MemberIdentifier('LifecycleClass0', 'closureSource');
        $closureTarget = new MemberIdentifier('LifecycleClass0', 'closureTarget');
        $fileClosureTarget = new MemberIdentifier('LifecycleClass0', 'fileClosureTarget');
        $fileArrowTarget = new MemberIdentifier('LifecycleClass0', 'fileArrowTarget');
        $referencesToAction = $project->references->getReferencesTo($frameworkAction);
        $referencesToLateAction = $project->references->getReferencesTo($lateFrameworkAction);
        $referencesToTopLevelValue = $project->references->getReferencesTo($topLevelValue);
        $referencesToTopLevelProperty = $project->references->getReferencesTo($topLevelProperty);
        $referencesToClosureSource = $project->references->getReferencesTo($closureSource);
        $referencesToClosureTarget = $project->references->getReferencesTo($closureTarget);
        $referencesToFileClosureTarget = $project->references->getReferencesTo($fileClosureTarget);
        $referencesToFileArrowTarget = $project->references->getReferencesTo($fileArrowTarget);
        $referencesToTopLevelFunction = $project->references->getReferencesTo('lifecycle_function_0');
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
            || count($referencesToLateAction) !== ($lateReferenceEnabled ? 1 : 0)
            || $lateReferenceEnabled
            && (
                !$referencesToLateAction[0]->source->symbol instanceof MemberIdentifier
                || $referencesToLateAction[0]->source->symbol->class !== 'lifecycleclass5'
                || $referencesToLateAction[0]->source->symbol->member !== 'value'
            )
        ) {
            throw new RuntimeException('The final merged native and synthetic references did not round-trip.');
        }

        if (
            count($referencesToTopLevelValue) !== 1
            || $referencesToTopLevelValue[0]->source->file !== 'src/file0.php'
            || $referencesToTopLevelValue[0]->kind !== ReferenceKind::Body
            || count($referencesToClosureSource) !== 1
            || $referencesToClosureSource[0]->source->file !== 'src/file0.php'
            || count($referencesToClosureTarget) !== 1
            || !$referencesToClosureTarget[0]->source->symbol instanceof MemberIdentifier
            || $referencesToClosureTarget[0]->source->symbol->class !== 'lifecycleclass0'
            || $referencesToClosureTarget[0]->source->symbol->member !== 'closuresource'
            || count($referencesToFileClosureTarget) !== 1
            || $referencesToFileClosureTarget[0]->source->file !== 'src/file0.php'
            || count($referencesToFileArrowTarget) !== 1
            || $referencesToFileArrowTarget[0]->source->file !== 'src/file0.php'
            || count($referencesToTopLevelFunction) !== 1
            || $referencesToTopLevelFunction[0]->source->file !== 'src/file0.php'
        ) {
            throw new RuntimeException('Native references did not retain their file and enclosing-symbol origins.');
        }

        $propertyKinds = [];
        foreach ($referencesToTopLevelProperty as $reference) {
            if ($reference->source->file !== 'src/file0.php') {
                throw new RuntimeException('A top-level property reference did not retain its file origin.');
            }

            $propertyKinds[] = $reference->kind->name;
        }
        if (
            count($referencesToTopLevelProperty) !== 3
            || !in_array(ReferenceKind::Body->name, $propertyKinds, true)
            || !in_array(ReferenceKind::PropertyRead->name, $propertyKinds, true)
            || !in_array(ReferenceKind::PropertyWrite->name, $propertyKinds, true)
        ) {
            throw new RuntimeException('Top-level property references did not retain their semantic kinds.');
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

    /** @mago-expect lint:halstead */
    private function verifySharedContext(LifecycleContext $context): ClassLikeMetadata
    {
        [$base, $missing] = $context->codebase->getMultipleClasses(['LifecycleClass0', 'DefinitelyMissing']);
        $extensionClass = $context->codebase->getClass('ExtensionProvided');
        $extensionFunction = $context->codebase->getFunction('extension_answer');
        $extensionProperty = $context->codebase->getProperty('ExtensionProvided', '$value');
        $inheritedAnswerMethod = $context->codebase->getDeclaringMethod('LifecycleClass0', 'answer');
        [$assertingFunction, $answerMethod, $missingFunctionLike] = $context->codebase->getMultipleFunctionLikes([
            new FunctionLikeIdentifier(IdentifierKind::Function_, 'lifecycle_assertions'),
            new FunctionLikeIdentifier(IdentifierKind::Method, 'answer', 'ExtensionProvided'),
            new FunctionLikeIdentifier(IdentifierKind::Closure, '{closure:missing.php:1:1}'),
        ]);
        if ($base === null || $missing !== null || !$context->codebase->classExists('LifecycleClass0')) {
            throw new RuntimeException('A lifecycle hook cannot query host classes.');
        }

        if (!$context->types->isContainedBy(Type::literalInt(1), Type::int())) {
            throw new RuntimeException('A lifecycle hook cannot compare types.');
        }
        $comparisons = [
            TypeComparison::equal(Type::int(), Type::int()),
            TypeComparison::containedBy(Type::literalInt(1), Type::int()),
            TypeComparison::canBeIdentical(Type::int(), Type::string()),
            TypeComparison::containedBy(Type::literalInt(1), Type::int()),
        ];
        if ($context->types->compareMultiple($comparisons) !== [true, true, false, true]) {
            throw new RuntimeException('A lifecycle hook cannot batch mixed or duplicate type comparisons.');
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

        if (
            $extensionFunction === null
            || count($extensionFunction->templates) !== 1
            || $extensionFunction->identifier->kind !== IdentifierKind::Function_
            || $extensionFunction->identifier->name !== 'extension_answer'
        ) {
            throw new RuntimeException('A lifecycle hook cannot query an external-stub function.');
        }

        $assertion = $assertingFunction?->assertions['$value'][0] ?? null;
        $ifTrueAssertion = $assertingFunction?->ifTrueAssertions['$text'][0] ?? null;
        $ifFalseAssertion = $assertingFunction?->ifFalseAssertions['$fallback'][0] ?? null;
        $methodAttribute = $answerMethod?->attributes[0] ?? null;
        $methodValue = $methodAttribute?->getArgument(0);
        $methodEnabled = $methodAttribute?->getArgument(1, 'enabled');
        $propertyAttribute = $extensionProperty?->attributes[0] ?? null;
        $propertyValue = $propertyAttribute?->getArgument(0);
        if (
            $assertingFunction === null
            || $answerMethod === null
            || $answerMethod->kind !== MetadataFunctionLikeKind::Method
            || $answerMethod->identifier->kind !== IdentifierKind::Method
            || $answerMethod->identifier->class !== 'ExtensionProvided'
            || $answerMethod->identifier->name !== 'answer'
            || $inheritedAnswerMethod === null
            || !$inheritedAnswerMethod->identifier->equals($answerMethod->identifier)
            || $methodAttribute?->name !== 'ExtensionMarker'
            || $methodValue === null
            || $methodValue->valueType?->getLiteralString() !== 'ExtensionProvided'
            || $methodValue->valueLocation === null
            || $methodEnabled === null
            || $methodEnabled->name !== 'enabled'
            || $methodEnabled->nameLocation === null
            || $methodEnabled->valueType?->getLiteralBool() !== false
            || $propertyValue?->valueType?->getLiteralString() !== 'property'
            || $missingFunctionLike !== null
            || $assertingFunction->assertionsInferred
            || !$assertion instanceof TypeAssertion
            || $assertion->kind !== TypeAssertionKind::IsType
            || !$context->types->equals($assertion->type, Type::int())
            || !$ifTrueAssertion instanceof TypeAssertion
            || $ifTrueAssertion->kind !== TypeAssertionKind::IsType
            || !$context->types->equals($ifTrueAssertion->type, Type::nonEmptyString())
            || !$ifFalseAssertion instanceof TypeAssertion
            || $ifFalseAssertion->kind !== TypeAssertionKind::IsType
            || !$context->types->equals($ifFalseAssertion->type, Type::null())
        ) {
            throw new RuntimeException('Function-like assertions or identifier lookup did not round-trip.');
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
final class LifecycleMethodCallHook implements MethodCallAnalysisHook
{
    public function __construct(
        private readonly string $plugin,
        private readonly string $auditLog,
    ) {}

    public function getTargets(): array
    {
        return [MethodTarget::exact('ExtensionProvided', 'answer')];
    }

    public function getRequirements(): array
    {
        return [FileAnalysisRequirement::ReceiverType, FileAnalysisRequirement::TargetSubtree];
    }

    public function analyze(NodeAnalysisContext $context): void
    {
        $call = CallExpression::fromNode($context->source, $context->node);
        if ($context->node->kind !== NodeKind::MethodCall || $call->getName($context->source) !== 'answer') {
            throw new RuntimeException('A targeted method-call hook received an unrelated call.');
        }
        if ($context->receiverType === null || !str_contains((string) $context->receiverType, 'LifecycleClass0')) {
            throw new RuntimeException('A targeted method-call hook did not receive its requested receiver type.');
        }

        $record = json_encode([
            $this->plugin,
            'method-call',
            $context->analysis->file,
            getmypid(),
        ], JSON_THROW_ON_ERROR);
        if (file_put_contents($this->auditLog, $record . "\n", FILE_APPEND | LOCK_EX) === false) {
            throw new RuntimeException('Unable to append to the lifecycle audit log.');
        }
    }
}

/**
 * @mago-expect lint:single-class-per-file
 */
final class LifecycleClassLikeHook implements ClassLikeAnalysisHook
{
    public function __construct(
        private readonly string $plugin,
        private readonly string $auditLog,
    ) {}

    public function getTargets(): array
    {
        return [ClassLikeTarget::descendantsOf('ExtensionProvided')];
    }

    public function getRequirements(): array
    {
        return [FileAnalysisRequirement::TargetSubtree, FileAnalysisRequirement::SourceText];
    }

    public function analyze(NodeAnalysisContext $context): void
    {
        if (
            $context->node->kind !== NodeKind::Class_
            || !str_contains($context->source->getText($context->node->span), 'class LifecycleClass0')
        ) {
            throw new RuntimeException('A descendant class-like hook received an unrelated declaration.');
        }

        $record = json_encode([
            $this->plugin,
            'class-like',
            $context->analysis->file,
            getmypid(),
        ], JSON_THROW_ON_ERROR);
        if (file_put_contents($this->auditLog, $record . "\n", FILE_APPEND | LOCK_EX) === false) {
            throw new RuntimeException('Unable to append to the lifecycle audit log.');
        }
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
