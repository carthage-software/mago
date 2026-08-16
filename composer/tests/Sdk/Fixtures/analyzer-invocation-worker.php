<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Fixtures;

use Mago\Sdk\Analyzer\CallableSignatureOverride;
use Mago\Sdk\Analyzer\CallableSignatureProviderContext;
use Mago\Sdk\Analyzer\ClassInitializerProvider;
use Mago\Sdk\Analyzer\ClassInitializerProviderContext;
use Mago\Sdk\Analyzer\ClassTarget;
use Mago\Sdk\Analyzer\EffectiveCallableSignature;
use Mago\Sdk\Analyzer\FunctionReturnTypeProvider;
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\InvocationKind;
use Mago\Sdk\Analyzer\IssueFilterContext;
use Mago\Sdk\Analyzer\IssueFilterDecision;
use Mago\Sdk\Analyzer\IssueFilterHook;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\MethodReturnTypeProvider;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\PropertyAccessKind;
use Mago\Sdk\Analyzer\PropertyInitializationProvider;
use Mago\Sdk\Analyzer\PropertyInitializationProviderContext;
use Mago\Sdk\Analyzer\PropertyTarget;
use Mago\Sdk\Analyzer\PropertyType;
use Mago\Sdk\Analyzer\PropertyTypeProvider;
use Mago\Sdk\Analyzer\PropertyTypeProviderContext;
use Mago\Sdk\Analyzer\ReturnTypeProviderContext;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\CallableParameter;
use Mago\Sdk\Analyzer\Type\NamedObjectType;
use Mago\Sdk\Analyzer\TypeComparison;
use Mago\Sdk\Extension;
use Mago\Sdk\Reporting\Level;
use Mago\Sdk\Worker;
use RuntimeException;

use function count;
use function dirname;
use function file_put_contents;
use function getenv;
use function is_string;
use function sort;
use function str_contains;
use function strtolower;

use const FILE_APPEND;
use const LOCK_EX;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

final class InvocationAudit
{
    public static function record(string $case): void
    {
        $path = getenv('MAGO_INVOCATION_AUDIT_LOG');
        if (
            !is_string($path)
            || $path === ''
            || file_put_contents($path, $case . "\n", FILE_APPEND | LOCK_EX) === false
        ) {
            throw new RuntimeException('Could not record an external invocation provider call.');
        }
    }
}

/**
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:single-class-per-file
 */
final class InvocationFunctionProvider implements FunctionReturnTypeProvider, CallableSignatureOverride
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('external_function'), FunctionTarget::exact('collect')];
    }

    public function getCallableSignature(CallableSignatureProviderContext $context): ?EffectiveCallableSignature
    {
        if ($context->invocation->name !== 'collect') {
            return null;
        }

        $argument = $context->invocation->arguments[0] ?? throw new RuntimeException(
            'The function signature proof received no argument.',
        );
        if ($argument->type !== null || $argument->expression !== "'function'") {
            throw new RuntimeException('A callable signature provider received an analyzed or incorrect argument.');
        }

        InvocationAudit::record('function-signature');

        return new EffectiveCallableSignature([new CallableParameter('$value', Type::string())]);
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $invocation = $context->invocation;
        if (
            $invocation->kind !== InvocationKind::Function
            || $invocation->declaringClass !== null
            || $invocation->receiverType !== null
        ) {
            throw new RuntimeException('A function provider received method invocation context.');
        }

        if ($invocation->name === 'collect') {
            $argument = $invocation->arguments[0]->type ?? throw new RuntimeException(
                'The function signature proof return provider received an untyped argument.',
            );
            InvocationAudit::record('function-signature-return');

            return $argument;
        }

        InvocationAudit::record('function');

        return Type::string();
    }
}

/**
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:halstead
 * @mago-expect lint:kan-defect
 * @mago-expect lint:single-class-per-file
 */
final class InvocationMethodProvider implements MethodReturnTypeProvider, CallableSignatureOverride
{
    public function getTargets(): array
    {
        return [
            MethodTarget::exact('BaseModel', 'query'),
            MethodTarget::exact('BaseModel', 'echoArgument'),
            MethodTarget::exact('BaseModel', 'acceptString'),
            MethodTarget::exact('BaseModel', 'late'),
            MethodTarget::exact('BaseModel', 'magicValues'),
            MethodTarget::exact('BaseModel', 'newQuery'),
            MethodTarget::exact('BaseModel', 'passthrough'),
            MethodTarget::exact('Builder', 'first'),
            MethodTarget::exact('DynamicProxy', 'dynamic'),
            MethodTarget::exact('DynamicProxy', 'acceptString'),
            MethodTarget::exact('DynamicFacade', 'dynamic'),
            MethodTarget::exact('Artisan', 'command'),
            MethodTarget::allMethods('Relation'),
            MethodTarget::exact('ExternalInterface', 'fetch'),
            MethodTarget::exact('ExternalService', 'resolve'),
            MethodTarget::exact('ExternalStaticService', 'resolve'),
            MethodTarget::exact('DeclinedContract', 'missing'),
        ];
    }

    public function getCallableSignature(CallableSignatureProviderContext $context): ?EffectiveCallableSignature
    {
        $invocation = $context->invocation;
        if ($invocation->declaringClass === 'Artisan' && $invocation->name === 'command') {
            InvocationAudit::record('closure-this-signature');

            return new EffectiveCallableSignature([
                new CallableParameter('$signature', Type::string()),
                new CallableParameter(
                    name: '$callback',
                    type: Type::namedObject('Closure'),
                    closureThisType: Type::namedObject('ClosureCommand'),
                ),
            ]);
        }

        if ($invocation->declaringClass === 'ExternalInterface' && $invocation->name === 'fetch') {
            $receiver = $invocation->receiverType ?? throw new RuntimeException(
                'The unresolved interface method received no receiver type.',
            );

            if (
                $invocation->kind !== InvocationKind::InstanceMethod
                || $invocation->arguments !== []
                || !$context->types->equals($receiver, Type::namedObject('ExternalInterface'))
            ) {
                throw new RuntimeException('The unresolved interface method received incorrect invocation context.');
            }

            InvocationAudit::record('missing-interface-signature');

            return new EffectiveCallableSignature([]);
        }

        if (
            (
                $invocation->declaringClass === 'ExternalService'
                || $invocation->declaringClass === 'ExternalStaticService'
            )
            && $invocation->name === 'resolve'
        ) {
            $declaringClass = $invocation->declaringClass;
            $receiver = $invocation->receiverType ?? throw new RuntimeException(
                'The unresolved class method received no receiver type.',
            );
            $argument = $invocation->arguments[0] ?? throw new RuntimeException(
                'The unresolved class method received no argument.',
            );
            $static = $declaringClass === 'ExternalStaticService';
            if (
                $argument->type !== null
                || $argument->expression !== ($static ? "'static-provider'" : "'instance-provider'")
                || $invocation->kind !== ($static ? InvocationKind::StaticMethod : InvocationKind::InstanceMethod)
                || !$context->types->equals($receiver, Type::namedObject($declaringClass))
            ) {
                throw new RuntimeException('The unresolved class method received incorrect invocation context.');
            }

            InvocationAudit::record($static ? 'missing-static-signature' : 'missing-class-signature');

            return new EffectiveCallableSignature([new CallableParameter('$value', Type::string())]);
        }

        if ($invocation->name !== 'acceptstring') {
            return null;
        }

        $receiver = $invocation->receiverType ?? throw new RuntimeException(
            'The method signature proof received no receiver type.',
        );
        $argument = $invocation->arguments[0] ?? throw new RuntimeException(
            'The method signature proof received no argument.',
        );
        $dynamic = $invocation->declaringClass === 'DynamicProxy';
        if (
            $argument->type !== null
            || ($dynamic ? $argument->expression !== "'dynamic-signature'" : $argument->expression !== "'method'")
        ) {
            throw new RuntimeException('A method signature provider received incomplete pre-analysis context.');
        }

        if ($dynamic) {
            if (
                $invocation->kind !== InvocationKind::InstanceMethod
                || !$context->types->equals($receiver, Type::namedObject('DynamicProxy'))
            ) {
                throw new RuntimeException('A dynamic method signature provider received incorrect receiver context.');
            }
            InvocationAudit::record('dynamic-signature');

            return new EffectiveCallableSignature([new CallableParameter('$value', Type::string())]);
        }

        if (
            $invocation->kind !== InvocationKind::StaticMethod
            || $invocation->declaringClass !== 'BaseModel'
            || !$context->types->equals($receiver, Type::namedObject('User'))
        ) {
            throw new RuntimeException('A declared method signature provider received incorrect receiver context.');
        }

        InvocationAudit::record('method-signature');

        return new EffectiveCallableSignature([new CallableParameter('$value', Type::string())]);
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $invocation = $context->invocation;
        $receiver = $invocation->receiverType ?? throw new RuntimeException(
            'A method provider received no receiver type.',
        );
        $named = self::named($receiver);

        if ($invocation->declaringClass === 'Artisan' && $invocation->name === 'command') {
            self::assertInvocation($context, InvocationKind::StaticMethod, 'Artisan');
            InvocationAudit::record('closure-this-return');

            return Type::void();
        }

        if ($invocation->declaringClass === 'CustomRelation' && $invocation->name === 'where') {
            self::assertInvocation($context, InvocationKind::InstanceMethod, 'CustomRelation');
            self::assertNamed($named, 'CustomRelation', 0, 0);
            InvocationAudit::record('relation-subclass');

            return Type::string();
        }

        if ($invocation->declaringClass === 'ExternalInterface' && $invocation->name === 'fetch') {
            self::assertInvocation($context, InvocationKind::InstanceMethod, 'ExternalInterface');
            self::assertNamed($named, 'ExternalInterface', 0, 0);
            InvocationAudit::record('missing-interface-return');

            return Type::namedObject('ExternalResult');
        }

        if (
            (
                $invocation->declaringClass === 'ExternalService'
                || $invocation->declaringClass === 'ExternalStaticService'
            )
            && $invocation->name === 'resolve'
        ) {
            $declaringClass = $invocation->declaringClass;

            $static = $declaringClass === 'ExternalStaticService';
            self::assertInvocation(
                $context,
                $static ? InvocationKind::StaticMethod : InvocationKind::InstanceMethod,
                $declaringClass,
            );

            self::assertNamed($named, $declaringClass, 0, 0);
            $argumentType = $invocation->arguments[0]->type ?? throw new RuntimeException(
                'The unresolved method return provider received an untyped argument.',
            );

            self::assertEqual(
                $context,
                $argumentType,
                Type::literalString($static ? 'static-provider' : 'instance-provider'),
            );

            InvocationAudit::record($static ? 'missing-static-return' : 'missing-class-return');

            return $argumentType;
        }

        if ($invocation->name === 'query') {
            self::assertInvocation($context, InvocationKind::StaticMethod, 'BaseModel');
            self::assertNamed($named, 'User', 0, 0);
            self::assertEqual($context, $receiver, Type::namedObject('User'));
            $magic = $context->codebase->getMagicProperty('BaseModel', '$magic');
            $inheritedMagic = $context->codebase->getDeclaringMagicProperty('User', '$magic');
            if (
                $magic === null
                || $magic->type === null
                || $inheritedMagic === null
                || $inheritedMagic->type === null
            ) {
                throw new RuntimeException('Magic property metadata did not preserve its type.');
            }
            if (
                !$context->codebase->magicPropertyExists('User', '$magic')
                || $context->codebase->propertyExists('BaseModel', '$magic')
                || $context->codebase->getProperty('BaseModel', '$magic') !== null
                || $context->codebase->checkMultipleMagicPropertiesExist([
                    new MemberIdentifier('User', '$magic'),
                    new MemberIdentifier('User', '$missing'),
                ]) !== [true, false]
                || !$context->types->equals($magic->type->type, Type::string())
                || !$context->types->equals($inheritedMagic->type->type, Type::string())
            ) {
                throw new RuntimeException(
                    'Magic property metadata did not round-trip independently of real properties.',
                );
            }
            InvocationAudit::record('static');

            return Type::namedObject('Builder', Type::namedObject('User'));
        }

        if ($invocation->name === 'newquery') {
            self::assertInvocation($context, InvocationKind::InstanceMethod, 'BaseModel');
            self::assertNamed($named, 'User', 0, 0);
            self::assertEqual($context, $receiver, Type::namedObject('User'));
            InvocationAudit::record('instance');

            return Type::namedObject('Builder', Type::namedObject('User'));
        }

        if ($invocation->name === 'echoargument') {
            self::assertInvocation($context, InvocationKind::StaticMethod, 'BaseModel');
            self::assertNamed($named, 'User', 0, 0);
            $argument = $invocation->arguments[0] ?? throw new RuntimeException(
                'The argument-handle proof received no argument.',
            );
            $argumentType = $argument->type ?? throw new RuntimeException(
                'The argument-handle proof received an untyped argument.',
            );
            self::assertEqual($context, $argumentType, Type::literalString('value'));
            InvocationAudit::record('argument-handle');

            return $argumentType;
        }

        if ($invocation->declaringClass === 'BaseModel' && $invocation->name === 'acceptstring') {
            self::assertInvocation($context, InvocationKind::StaticMethod, 'BaseModel');
            self::assertNamed($named, 'User', 0, 0);
            $argumentType = $invocation->arguments[0]->type ?? throw new RuntimeException(
                'The method signature proof return provider received an untyped argument.',
            );
            self::assertEqual($context, $argumentType, Type::literalString('method'));
            InvocationAudit::record('method-signature-return');

            return $argumentType;
        }

        if ($invocation->name === 'late') {
            self::assertInvocation($context, InvocationKind::StaticMethod, 'BaseModel');
            self::assertLateStaticNamed($named, 'BaseModel');
            InvocationAudit::record('late-static');

            return $receiver;
        }

        if ($invocation->name === 'magicvalues') {
            self::assertInvocation($context, InvocationKind::StaticMethod, 'BaseModel');
            self::assertNamed($named, 'User', 0, 0);
            $magicProperty = $context->codebase->getDeclaringMagicProperty(
                'User',
                '$magic',
            ) ?? throw new RuntimeException('The inherited magic property is unavailable.');
            $magicTypeMetadata = $magicProperty->type ?? throw new RuntimeException(
                'The inherited magic property lost its complete type.',
            );
            $magicType = $magicTypeMetadata->type;
            InvocationAudit::record('complete-composite');

            return Type::list(Type::namedObject('Builder', $magicType));
        }

        if ($invocation->name === 'first') {
            self::assertInvocation($context, InvocationKind::InstanceMethod, 'Builder');
            self::assertNamed($named, 'Builder', 1, 0);
            self::assertEqual($context, $receiver, Type::namedObject('Builder', Type::namedObject('User')));
            $parameter = $named->parameters[0] ?? throw new RuntimeException(
                'The generic receiver lost its parameter.',
            );
            self::assertNamed(self::named($parameter), 'User', 0, 0);
            InvocationAudit::record('generic');

            return $parameter;
        }

        if ($invocation->name === 'passthrough') {
            self::assertInvocation($context, InvocationKind::InstanceMethod, 'BaseModel');
            $names = [strtolower($named->name)];
            foreach ($named->intersections ?? [] as $intersection) {
                if (!$intersection instanceof NamedObjectType) {
                    continue;
                }

                $names[] = strtolower($intersection->name);
            }
            sort($names);
            if ($names !== ['basemodel', 'marker']) {
                throw new RuntimeException('The intersected receiver did not survive the binary round trip.');
            }
            InvocationAudit::record('intersection');

            return $receiver;
        }

        if ($invocation->declaringClass === 'DynamicProxy' && $invocation->name === 'dynamic') {
            self::assertInvocation($context, InvocationKind::InstanceMethod, 'DynamicProxy');
            self::assertNamed($named, 'DynamicProxy', 0, 0);
            $argumentType = $invocation->arguments[0]->type ?? throw new RuntimeException(
                'The dynamic instance invocation lost its argument type.',
            );
            self::assertEqual($context, $argumentType, Type::literalString('instance'));
            InvocationAudit::record('dynamic-instance');

            return Type::union($argumentType, Type::string());
        }

        if ($invocation->declaringClass === 'DynamicProxy' && $invocation->name === 'acceptstring') {
            self::assertInvocation($context, InvocationKind::InstanceMethod, 'DynamicProxy');
            self::assertNamed($named, 'DynamicProxy', 0, 0);
            $argumentType = $invocation->arguments[0]->type ?? throw new RuntimeException(
                'The dynamic signature proof lost its analyzed argument type.',
            );
            self::assertEqual($context, $argumentType, Type::literalString('dynamic-signature'));
            InvocationAudit::record('dynamic-signature-return');

            return $argumentType;
        }

        if ($invocation->declaringClass === 'DynamicFacade' && $invocation->name === 'dynamic') {
            self::assertInvocation($context, InvocationKind::StaticMethod, 'DynamicFacade');
            self::assertNamed($named, 'DynamicFacade', 0, 0);
            $argumentType = $invocation->arguments[0]->type ?? throw new RuntimeException(
                'The dynamic static invocation lost its argument type.',
            );
            self::assertEqual($context, $argumentType, Type::literalString('static'));
            InvocationAudit::record('dynamic-static');

            return Type::array(Type::int(), $argumentType);
        }

        throw new RuntimeException("Unexpected method invocation `{$invocation->name}`.");
    }

    private static function assertInvocation(
        ReturnTypeProviderContext $context,
        InvocationKind $kind,
        string $declaringClass,
    ): void {
        $invocation = $context->invocation;
        if ($invocation->kind !== $kind || $invocation->declaringClass !== $declaringClass) {
            throw new RuntimeException('The invocation kind or declaring class was not preserved.');
        }
    }

    private static function assertEqual(ReturnTypeProviderContext $context, Type $actual, Type $expected): void
    {
        $comparisons = [
            TypeComparison::equal($actual, $expected),
            TypeComparison::containedBy($actual, $expected),
            TypeComparison::canBeIdentical($actual, $expected),
            TypeComparison::equal($actual, $expected),
        ];
        if ($context->types->compareMultiple($comparisons) !== [true, true, true, true]) {
            throw new RuntimeException('The received receiver type is not usable with TypeComparator.');
        }
    }

    private static function named(Type $type): NamedObjectType
    {
        if (count($type->atomicTypes) !== 1 || !$type->atomicTypes[0] instanceof NamedObjectType) {
            throw new RuntimeException('Expected exactly one named receiver type.');
        }

        return $type->atomicTypes[0];
    }

    private static function assertNamed(NamedObjectType $type, string $name, int $parameters, int $intersections): void
    {
        if (
            strtolower($type->name) !== strtolower($name)
            || count($type->parameters ?? []) !== $parameters
            || count($type->intersections ?? []) !== $intersections
            || $type->static
        ) {
            throw new RuntimeException("Expected complete named object type `{$name}`.");
        }
    }

    private static function assertLateStaticNamed(NamedObjectType $type, string $name): void
    {
        if (
            strtolower($type->name) !== strtolower($name)
            || $type->parameters !== null
            || $type->intersections !== null
            || !$type->static
            || !$type->isThis
        ) {
            $parameters = $type->parameters === null ? 'null' : (string) count($type->parameters);
            $intersections = $type->intersections === null ? 'null' : (string) count($type->intersections);
            $static = $type->static ? 'true' : 'false';
            $isThis = $type->isThis ? 'true' : 'false';
            throw new RuntimeException(
                "Expected complete late-static named object type `{$name}`; received `{$type->name}` "
                . "with {$parameters} parameters, {$intersections} intersections, static={$static}, this={$isThis}.",
            );
        }
    }
}

/**
 * @mago-expect lint:single-class-per-file
 */
final class InvocationPropertyProvider implements PropertyTypeProvider
{
    public function getTargets(): array
    {
        return [PropertyTarget::allProperties('BaseModel')];
    }

    public function getPropertyType(PropertyTypeProviderContext $context): ?PropertyType
    {
        $access = $context->access;
        if ($access->class !== 'User' || !$context->types->equals($access->receiverType, Type::namedObject('User'))) {
            throw new RuntimeException('A property provider received incorrect subclass or receiver context.');
        }

        $operation = match ($access->kind) {
            PropertyAccessKind::Read => 'read',
            PropertyAccessKind::Write => 'write',
        };
        InvocationAudit::record("property-{$access->property}-{$operation}");

        return match ($access->property) {
            'name' => new PropertyType(Type::string(), Type::int()),
            'id' => new PropertyType(readType: Type::int()),
            'secret' => new PropertyType(writeType: Type::string()),
            'self' => new PropertyType(readType: $access->receiverType),
            default => null,
        };
    }
}

/**
 * @mago-expect lint:single-class-per-file
 */
final class InvocationPropertyInitializationProvider implements PropertyInitializationProvider
{
    public function getTargets(): array
    {
        return [PropertyTarget::exact('FrameworkTestCase', 'application')];
    }

    public function isPropertyInitialized(PropertyInitializationProviderContext $context): bool
    {
        $declaredType = $context->property->declaredType?->type;
        if (
            $context->declaringClass !== 'ManagedTestCase'
            || $context->property->name !== '$application'
            || $declaredType === null
            || !$context->types->equals($declaredType, Type::namedObject('ExternalResult'))
            || $context->codebase->getClass($context->declaringClass) === null
        ) {
            throw new RuntimeException('A property initialization provider received incomplete declared metadata.');
        }

        if (getenv('MAGO_INVOCATION_DECLINE_PROPERTY_INITIALIZATION') === '1') {
            return false;
        }

        InvocationAudit::record('property-initialization');
        return true;
    }
}

/**
 * @mago-expect lint:single-class-per-file
 */
final class InvocationClassInitializerProvider implements ClassInitializerProvider
{
    public function getTargets(): array
    {
        return [ClassTarget::exact('FrameworkTestCase')];
    }

    public function getClassInitializers(ClassInitializerProviderContext $context): array
    {
        if (!$context->codebase->methodExists($context->class->name, 'setUp')) {
            return [];
        }

        InvocationAudit::record('class-initializer-' . strtolower($context->class->name));

        return ['setUp'];
    }
}

/**
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:single-class-per-file
 */
final class InvocationIssueFilterHook implements IssueFilterHook
{
    public function getCodes(): array
    {
        return ['non-existent-function'];
    }

    public function filterIssue(IssueFilterContext $context): IssueFilterDecision
    {
        if (
            $context->file !== 'src/invocation.php'
            || !str_contains($context->contents, 'issue-filter-batch-proof')
            || !$context->codebase->functionExists('known_function')
            || !$context->types->equals(Type::string(), Type::string())
            || $context->issue->level !== Level::Error
            || $context->issue->annotations === []
        ) {
            throw new RuntimeException('An issue-filter hook received incomplete file or diagnostic context.');
        }

        InvocationAudit::record('issue-filter-' . $context->issue->message);
        if (
            $context->issue->code === 'non-existent-function'
            && str_contains($context->issue->message, '`filtered_missing`')
        ) {
            return IssueFilterDecision::Remove;
        }

        return IssueFilterDecision::Keep;
    }
}

/**
 * @mago-expect lint:single-class-per-file
 */
final class InvocationPlugin implements Plugin
{
    public function __construct(
        private readonly bool $registerMethodProvider,
        private readonly bool $registerIssueFilter,
    ) {}

    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            'invocation-proof',
            'Invocation proof',
            'Validates complete provider invocation context.',
        );
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerFunctionReturnTypeProvider(new InvocationFunctionProvider());
        if ($this->registerMethodProvider) {
            $registry->registerMethodReturnTypeProvider(new InvocationMethodProvider());
        }
        $registry->registerPropertyTypeProvider(new InvocationPropertyProvider());
        $registry->registerPropertyInitializationProvider(new InvocationPropertyInitializationProvider());
        $registry->registerClassInitializerProvider(new InvocationClassInitializerProvider());
        if ($this->registerIssueFilter) {
            $registry->registerIssueFilterHook(new InvocationIssueFilterHook());
        }
    }
}

$registerMethodProvider = getenv('MAGO_INVOCATION_FUNCTION_ONLY') !== '1';
$registerIssueFilter = getenv('MAGO_INVOCATION_ISSUE_FILTER') === '1';

(new Worker(new Extension(
    identifier: 'mago/invocation-proof',
    name: 'Mago invocation proof',
    version: '1.0.0',
    analyzerPlugins: [new InvocationPlugin($registerMethodProvider, $registerIssueFilter)],
)))->run();
