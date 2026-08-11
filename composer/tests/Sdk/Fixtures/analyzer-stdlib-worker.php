<?php

declare(strict_types=1);

use Mago\Sdk\Analyzer\FunctionReturnTypeProvider;
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\Plugin;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PluginRegistry;
use Mago\Sdk\Analyzer\ReturnTypeProviderContext;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\ArrayItem;
use Mago\Sdk\Analyzer\Type\ArrayKey;
use Mago\Sdk\Analyzer\Type\ArrayKeyKind;
use Mago\Sdk\Analyzer\Type\KeyedArrayType;
use Mago\Sdk\Extension;
use Mago\Sdk\Worker;

require_once dirname(__DIR__, 4) . '/vendor/autoload.php';

final class StrlenProvider implements FunctionReturnTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('strlen')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $string = $context->invocation->getArgument(0, 'string')?->type;
        if ($string === null) {
            return null;
        }

        $literal = $string->getLiteralString();

        return $literal === null ? Type::nonNegativeInt() : Type::literalInt(strlen($literal));
    }
}

/** @mago-expect lint:single-class-per-file */
final class JsonEncodeProvider implements FunctionReturnTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('json_encode')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $flags = $context->invocation->getArgument(1, 'flags');
        if ($flags?->type === null) {
            return null;
        }

        $literal = $flags->type->getLiteralInt();
        $throws = $literal !== null && ($literal & JSON_THROW_ON_ERROR) !== 0;

        return $throws ? Type::nonEmptyString() : Type::union(Type::nonEmptyString(), Type::false());
    }
}

/**
 * @mago-expect lint:single-class-per-file
 * @mago-expect lint:cyclomatic-complexity
 */
final class RandomBytesProvider implements FunctionReturnTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('random_bytes')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $length = $context->invocation->getArgument(0, 'length')?->type;
        if ($length === null) {
            return null;
        }

        $literal = $length->getLiteralInt();
        if ($literal !== null) {
            return $literal > 0 ? Type::nonEmptyString() : Type::literalString('');
        }

        return null;
    }
}

/** @mago-expect lint:single-class-per-file */
final class AbsProvider implements FunctionReturnTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('abs'), FunctionTarget::exact('Psl\\Math\\abs')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $number = $context->invocation->getArgument(0, 'num')?->type;
        if ($number === null || !$context->types->isContainedBy($number, Type::int())) {
            return null;
        }

        $literal = $number->getLiteralInt();
        if ($literal !== null && $literal !== PHP_INT_MIN) {
            return Type::literalInt(abs($literal));
        }

        return Type::nonNegativeInt();
    }
}

/** @mago-expect lint:single-class-per-file */
final class CompleteTypeProvider implements FunctionReturnTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('mago_complete_type_fixture')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $context->cancellation->throwIfCancelled();

        return Type::fromAtomic(
            new KeyedArrayType(
                [new ArrayItem(new ArrayKey(ArrayKeyKind::String, 'value'), false, Type::literalString('ok'))],
                Type::string(),
                Type::mixed(),
                true,
            ),
        );
    }
}

/** @mago-expect lint:single-class-per-file */
final class NestedTypeComparisonProvider implements FunctionReturnTypeProvider
{
    public function getTargets(): array
    {
        return [FunctionTarget::exact('mago_nested_type_fixture')];
    }

    public function getReturnType(ReturnTypeProviderContext $context): ?Type
    {
        $type = $context->invocation->getArgument(0, 'value')?->type;
        $atomic = $type?->atomicTypes[0] ?? null;
        if (!$atomic instanceof KeyedArrayType || $atomic->knownItems === null) {
            return null;
        }

        $item = $atomic->knownItems[0] ?? null;
        if ($item === null) {
            return null;
        }

        return $context->types->isContainedBy($item->type, Type::int()) ? Type::true() : Type::false();
    }
}

/** @mago-expect lint:single-class-per-file */
final class ExternalStdlibPlugin implements Plugin
{
    public function getDefinition(): PluginDefinition
    {
        return new PluginDefinition(
            identifier: 'external-stdlib-benchmark',
            name: 'External stdlib benchmark',
            description: 'PHP ports of five native standard-library return-type providers.',
        );
    }

    public function register(PluginRegistry $registry): void
    {
        $registry->registerFunctionReturnTypeProvider(new StrlenProvider());
        $registry->registerFunctionReturnTypeProvider(new JsonEncodeProvider());
        $registry->registerFunctionReturnTypeProvider(new RandomBytesProvider());
        $registry->registerFunctionReturnTypeProvider(new AbsProvider());
        $registry->registerFunctionReturnTypeProvider(new CompleteTypeProvider());
        $registry->registerFunctionReturnTypeProvider(new NestedTypeComparisonProvider());
    }
}

$extension = new Extension(
    identifier: 'mago/external-stdlib-benchmark',
    name: 'External stdlib benchmark fixture',
    version: '1.0.0',
    analyzerPlugins: [new ExternalStdlibPlugin()],
);

(new Worker($extension))->run();
