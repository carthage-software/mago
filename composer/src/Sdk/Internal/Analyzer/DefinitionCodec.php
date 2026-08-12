<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Definition\ClassConstantDefinition;
use Mago\Sdk\Analyzer\Definition\ClassLikeDefinition;
use Mago\Sdk\Analyzer\Definition\ConstantDefinition;
use Mago\Sdk\Analyzer\Definition\EnumCaseDefinition;
use Mago\Sdk\Analyzer\Definition\FunctionDefinition;
use Mago\Sdk\Analyzer\Definition\MethodDefinition;
use Mago\Sdk\Analyzer\Definition\ParameterDefinition;
use Mago\Sdk\Analyzer\Definition\PropertyDefinition;
use Mago\Sdk\Analyzer\Definition\TemplateDefinition;
use Mago\Sdk\Analyzer\Metadata\ClassLikeKind;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\Variance;
use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 * @mago-expect lint:too-many-methods
 */
final class DefinitionCodec
{
    public static function writeClassLike(PayloadWriter $writer, ClassLikeDefinition $definition): void
    {
        $writer->writeBytes($definition->name);
        $writer->writeU8(match ($definition->kind) {
            ClassLikeKind::Class_ => 1,
            ClassLikeKind::Interface => 2,
            ClassLikeKind::Trait => 3,
            ClassLikeKind::Enum => 4,
        });
        $writer->writeU64($definition->flags);
        $writer->writeOptionalString($definition->parentClass);
        self::writeNames($writer, $definition->parentInterfaces);
        self::writeNames($writer, $definition->requiredExtends);
        self::writeNames($writer, $definition->requiredImplements);
        self::writeNames($writer, $definition->usedTraits);
        self::writeOptionalType($writer, $definition->enumType);
        self::writeTemplates($writer, $definition->templates);
        $writer->writeCount($definition->extendedTypes);
        foreach ($definition->extendedTypes as $parent => $types) {
            $writer->writeBytes($parent);
            $writer->writeCount($types);
            foreach ($types as $type) {
                TypeCodec::writeComplete($writer, $type);
            }
        }
        $writer->writeCount($definition->typeAliases);
        foreach ($definition->typeAliases as $name => $type) {
            $writer->writeBytes($name);
            TypeCodec::writeComplete($writer, $type);
        }
        $writer->writeCount($definition->mixins);
        foreach ($definition->mixins as $mixin) {
            TypeCodec::writeComplete($writer, $mixin);
        }
        self::writeOptionalBoolean($writer, $definition->sealedMethods);
        self::writeOptionalBoolean($writer, $definition->sealedProperties);
        $writer->writeBoolean($definition->permittedInheritors !== null);
        if ($definition->permittedInheritors !== null) {
            self::writeNames($writer, $definition->permittedInheritors);
        }
        $writer->writeCount($definition->methods);
        foreach ($definition->methods as $method) {
            self::writeMethod($writer, $method);
        }
        $writer->writeCount($definition->properties);
        foreach ($definition->properties as $property) {
            self::writeProperty($writer, $property);
        }
        $writer->writeCount($definition->magicProperties);
        foreach ($definition->magicProperties as $property) {
            self::writeProperty($writer, $property);
        }
        $writer->writeCount($definition->constants);
        foreach ($definition->constants as $constant) {
            self::writeClassConstant($writer, $constant);
        }
        $writer->writeCount($definition->enumCases);
        foreach ($definition->enumCases as $case) {
            self::writeEnumCase($writer, $case);
        }
    }

    /**
     * @mago-expect lint:halstead
     */
    public static function readClassLike(PayloadReader $reader): ClassLikeDefinition
    {
        $name = $reader->readBytes();
        $kind = self::readClassLikeKind($reader);
        $flags = $reader->readU64();
        $parent = $reader->readOptionalString();
        $interfaces = self::readNames($reader);
        $requiredExtends = self::readNames($reader);
        $requiredImplements = self::readNames($reader);
        $traits = self::readNames($reader);
        $enumType = self::readOptionalType($reader);
        $templates = self::readTemplates($reader);
        $extendedCount = $reader->readCount(65_536);
        $extendedTypes = [];
        for ($index = 0; $index < $extendedCount; ++$index) {
            $parent = $reader->readBytes();
            $typeCount = $reader->readCount(65_536);
            $types = [];
            for ($typeIndex = 0; $typeIndex < $typeCount; ++$typeIndex) {
                $types[] = TypeCodec::readComplete($reader);
            }
            $extendedTypes[$parent] = $types;
        }
        $aliasCount = $reader->readCount(65_536);
        $aliases = [];
        for ($index = 0; $index < $aliasCount; ++$index) {
            $aliases[$reader->readBytes()] = TypeCodec::readComplete($reader);
        }
        $mixinCount = $reader->readCount(65_536);
        $mixins = [];
        for ($index = 0; $index < $mixinCount; ++$index) {
            $mixins[] = TypeCodec::readComplete($reader);
        }
        $sealedMethods = self::readOptionalBoolean($reader);
        $sealedProperties = self::readOptionalBoolean($reader);
        $permittedInheritors = $reader->readBoolean() ? self::readNames($reader) : null;
        $methodCount = $reader->readCount(65_536);
        $methods = [];
        for ($index = 0; $index < $methodCount; ++$index) {
            $methods[] = self::readMethod($reader);
        }
        $propertyCount = $reader->readCount(65_536);
        $properties = [];
        for ($index = 0; $index < $propertyCount; ++$index) {
            $properties[] = self::readProperty($reader);
        }
        $magicPropertyCount = $reader->readCount(65_536);
        $magicProperties = [];
        for ($index = 0; $index < $magicPropertyCount; ++$index) {
            $magicProperties[] = self::readProperty($reader);
        }
        $constantCount = $reader->readCount(65_536);
        $constants = [];
        for ($index = 0; $index < $constantCount; ++$index) {
            $constants[] = self::readClassConstant($reader);
        }
        $caseCount = $reader->readCount(65_536);
        $cases = [];
        for ($index = 0; $index < $caseCount; ++$index) {
            $cases[] = self::readEnumCase($reader);
        }

        return new ClassLikeDefinition(
            $name,
            $kind,
            $parent,
            $interfaces,
            $requiredExtends,
            $requiredImplements,
            $traits,
            $enumType,
            $methods,
            $properties,
            $magicProperties,
            $constants,
            $cases,
            $templates,
            $extendedTypes,
            $aliases,
            $mixins,
            $sealedMethods,
            $sealedProperties,
            $permittedInheritors,
            $flags,
        );
    }

    public static function writeFunction(PayloadWriter $writer, FunctionDefinition $definition): void
    {
        self::writeFunctionLike(
            $writer,
            $definition->name,
            $definition->flags,
            $definition->parameters,
            $definition->nativeReturnType,
            $definition->returnType,
            $definition->throws,
            $definition->templates,
        );
    }

    public static function readFunction(PayloadReader $reader): FunctionDefinition
    {
        [$name, $flags, $parameters, $nativeReturn, $return, $throws, $templates] = self::readFunctionLike($reader);

        return new FunctionDefinition($name, $parameters, $return, $nativeReturn, $throws, $templates, $flags);
    }

    public static function writeConstant(PayloadWriter $writer, ConstantDefinition $definition): void
    {
        $writer->writeBytes($definition->name);
        $writer->writeU64($definition->flags);
        self::writeOptionalType($writer, $definition->type);
        self::writeOptionalType($writer, $definition->valueType);
    }

    public static function readConstant(PayloadReader $reader): ConstantDefinition
    {
        $name = $reader->readBytes();
        $flags = $reader->readU64();

        return new ConstantDefinition($name, self::readOptionalType($reader), self::readOptionalType($reader), $flags);
    }

    private static function writeMethod(PayloadWriter $writer, MethodDefinition $definition): void
    {
        self::writeFunctionLike(
            $writer,
            $definition->name,
            $definition->flags,
            $definition->parameters,
            $definition->nativeReturnType,
            $definition->returnType,
            $definition->throws,
            $definition->templates,
        );
        self::writeVisibility($writer, $definition->visibility);
    }

    private static function readMethod(PayloadReader $reader): MethodDefinition
    {
        [$name, $flags, $parameters, $nativeReturn, $return, $throws, $templates] = self::readFunctionLike($reader);

        return new MethodDefinition(
            $name,
            $parameters,
            $return,
            $nativeReturn,
            $throws,
            self::readVisibility($reader),
            $templates,
            $flags,
        );
    }

    /**
     * @param list<ParameterDefinition> $parameters
     * @param list<Type> $throws
     * @param list<TemplateDefinition> $templates
     * @mago-expect lint:excessive-parameter-list
     */
    private static function writeFunctionLike(
        PayloadWriter $writer,
        string $name,
        int $flags,
        array $parameters,
        ?Type $nativeReturn,
        ?Type $return,
        array $throws,
        array $templates,
    ): void {
        $writer->writeBytes($name);
        $writer->writeU64($flags);
        $writer->writeCount($parameters);
        foreach ($parameters as $parameter) {
            $writer->writeBytes($parameter->name);
            $writer->writeU64($parameter->flags);
            self::writeOptionalType($writer, $parameter->nativeType);
            self::writeOptionalType($writer, $parameter->type);
            self::writeOptionalType($writer, $parameter->outType);
            self::writeOptionalType($writer, $parameter->closureThisType);
            self::writeOptionalType($writer, $parameter->defaultType);
        }
        self::writeOptionalType($writer, $nativeReturn);
        self::writeOptionalType($writer, $return);
        $writer->writeCount($throws);
        foreach ($throws as $type) {
            TypeCodec::writeComplete($writer, $type);
        }
        self::writeTemplates($writer, $templates);
    }

    /**
     * @return array{string, int, list<ParameterDefinition>, Type|null, Type|null, list<Type>, list<TemplateDefinition>}
     */
    private static function readFunctionLike(PayloadReader $reader): array
    {
        $name = $reader->readBytes();
        $flags = $reader->readU64();
        $parameterCount = $reader->readCount(65_536);
        $parameters = [];
        for ($index = 0; $index < $parameterCount; ++$index) {
            $parameterName = $reader->readBytes();
            $parameterFlags = $reader->readU64();
            $parameterNativeType = self::readOptionalType($reader);
            $parameterType = self::readOptionalType($reader);
            $parameters[] = new ParameterDefinition(
                $parameterName,
                $parameterType,
                $parameterNativeType,
                self::readOptionalType($reader),
                self::readOptionalType($reader),
                self::readOptionalType($reader),
                $parameterFlags,
            );
        }
        $nativeReturn = self::readOptionalType($reader);
        $return = self::readOptionalType($reader);
        $throwCount = $reader->readCount(65_536);
        $throws = [];
        for ($index = 0; $index < $throwCount; ++$index) {
            $throws[] = TypeCodec::readComplete($reader);
        }
        $templates = self::readTemplates($reader);

        return [$name, $flags, $parameters, $nativeReturn, $return, $throws, $templates];
    }

    private static function writeProperty(PayloadWriter $writer, PropertyDefinition $definition): void
    {
        $writer->writeBytes($definition->name);
        $writer->writeU64($definition->flags);
        self::writeVisibility($writer, $definition->readVisibility);
        self::writeVisibility($writer, $definition->writeVisibility);
        self::writeOptionalType($writer, $definition->nativeType);
        self::writeOptionalType($writer, $definition->type);
        self::writeOptionalType($writer, $definition->writeType);
        self::writeOptionalType($writer, $definition->defaultType);
    }

    private static function readProperty(PayloadReader $reader): PropertyDefinition
    {
        $name = $reader->readBytes();
        $flags = $reader->readU64();
        $readVisibility = self::readVisibility($reader);
        $writeVisibility = self::readVisibility($reader);
        $nativeType = self::readOptionalType($reader);
        $type = self::readOptionalType($reader);

        return new PropertyDefinition(
            $name,
            $type,
            $nativeType,
            self::readOptionalType($reader),
            self::readOptionalType($reader),
            $readVisibility,
            $writeVisibility,
            $flags,
        );
    }

    private static function writeClassConstant(PayloadWriter $writer, ClassConstantDefinition $definition): void
    {
        $writer->writeBytes($definition->name);
        self::writeVisibility($writer, $definition->visibility);
        $writer->writeU64($definition->flags);
        self::writeOptionalType($writer, $definition->nativeType);
        self::writeOptionalType($writer, $definition->type);
        self::writeOptionalType($writer, $definition->valueType);
    }

    private static function readClassConstant(PayloadReader $reader): ClassConstantDefinition
    {
        $name = $reader->readBytes();
        $visibility = self::readVisibility($reader);
        $flags = $reader->readU64();
        $nativeType = self::readOptionalType($reader);
        $type = self::readOptionalType($reader);

        return new ClassConstantDefinition(
            $name,
            $type,
            $nativeType,
            self::readOptionalType($reader),
            $visibility,
            $flags,
        );
    }

    private static function writeEnumCase(PayloadWriter $writer, EnumCaseDefinition $definition): void
    {
        $writer->writeBytes($definition->name);
        $writer->writeU64($definition->flags);
        self::writeOptionalType($writer, $definition->valueType);
    }

    private static function readEnumCase(PayloadReader $reader): EnumCaseDefinition
    {
        $name = $reader->readBytes();
        $flags = $reader->readU64();

        return new EnumCaseDefinition($name, self::readOptionalType($reader), $flags);
    }

    /** @param list<string> $names */
    private static function writeNames(PayloadWriter $writer, array $names): void
    {
        $writer->writeCount($names);
        foreach ($names as $name) {
            $writer->writeBytes($name);
        }
    }

    /** @return list<string> */
    private static function readNames(PayloadReader $reader): array
    {
        $count = $reader->readCount(65_536);
        $names = [];
        for ($index = 0; $index < $count; ++$index) {
            $names[] = $reader->readBytes();
        }
        return $names;
    }

    /** @param list<TemplateDefinition> $templates */
    private static function writeTemplates(PayloadWriter $writer, array $templates): void
    {
        $writer->writeCount($templates);
        foreach ($templates as $template) {
            $writer->writeBytes($template->name);
            TypeCodec::writeComplete($writer, $template->constraint);
            self::writeOptionalType($writer, $template->default);
            $writer->writeU8(match ($template->variance) {
                Variance::Invariant => 1,
                Variance::Covariant => 2,
                Variance::Contravariant => 3,
                Variance::Bivariant => 4,
            });
            $writer->writeBoolean($template->readonly);
        }
    }

    /** @return list<TemplateDefinition> */
    private static function readTemplates(PayloadReader $reader): array
    {
        $count = $reader->readCount(65_536);
        $templates = [];
        for ($index = 0; $index < $count; ++$index) {
            $name = $reader->readBytes();
            $constraint = TypeCodec::readComplete($reader);
            $default = self::readOptionalType($reader);
            $variance = match ($value = $reader->readU8()) {
                1 => Variance::Invariant,
                2 => Variance::Covariant,
                3 => Variance::Contravariant,
                4 => Variance::Bivariant,
                default => throw new ProtocolException("Unknown template variance {$value}."),
            };
            $templates[] = new TemplateDefinition($name, $constraint, $default, $variance, $reader->readBoolean());
        }

        return $templates;
    }

    private static function writeOptionalType(PayloadWriter $writer, ?Type $type): void
    {
        $writer->writeBoolean($type !== null);
        if ($type !== null) {
            TypeCodec::writeComplete($writer, $type);
        }
    }

    private static function readOptionalType(PayloadReader $reader): ?Type
    {
        return $reader->readBoolean() ? TypeCodec::readComplete($reader) : null;
    }

    private static function writeOptionalBoolean(PayloadWriter $writer, ?bool $value): void
    {
        $writer->writeU8(match ($value) {
            null => 0,
            false => 1,
            true => 2,
        });
    }

    private static function readOptionalBoolean(PayloadReader $reader): ?bool
    {
        return match ($value = $reader->readU8()) {
            0 => null,
            1 => false,
            2 => true,
            default => throw new ProtocolException("Unknown optional boolean {$value}."),
        };
    }

    private static function writeVisibility(PayloadWriter $writer, Visibility $visibility): void
    {
        $writer->writeU8(match ($visibility) {
            Visibility::Public => 1,
            Visibility::Protected => 2,
            Visibility::Private => 3,
        });
    }

    private static function readVisibility(PayloadReader $reader): Visibility
    {
        return match ($value = $reader->readU8()) {
            1 => Visibility::Public,
            2 => Visibility::Protected,
            3 => Visibility::Private,
            default => throw new ProtocolException("Unknown visibility {$value}."),
        };
    }

    private static function readClassLikeKind(PayloadReader $reader): ClassLikeKind
    {
        return match ($value = $reader->readU8()) {
            1 => ClassLikeKind::Class_,
            2 => ClassLikeKind::Interface,
            3 => ClassLikeKind::Trait,
            4 => ClassLikeKind::Enum,
            default => throw new ProtocolException("Unknown class-like kind {$value}."),
        };
    }
}
