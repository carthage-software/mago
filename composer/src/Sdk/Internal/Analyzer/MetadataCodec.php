<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Metadata\AttributeMetadata;
use Mago\Sdk\Analyzer\Metadata\ClassConstantMetadata;
use Mago\Sdk\Analyzer\Metadata\ClassLikeKind;
use Mago\Sdk\Analyzer\Metadata\ClassLikeMetadata;
use Mago\Sdk\Analyzer\Metadata\ConstantMetadata;
use Mago\Sdk\Analyzer\Metadata\EnumCaseMetadata;
use Mago\Sdk\Analyzer\Metadata\FunctionLikeKind;
use Mago\Sdk\Analyzer\Metadata\FunctionLikeMetadata;
use Mago\Sdk\Analyzer\Metadata\MetadataFlags;
use Mago\Sdk\Analyzer\Metadata\ParameterMetadata;
use Mago\Sdk\Analyzer\Metadata\PropertyHookMetadata;
use Mago\Sdk\Analyzer\Metadata\PropertyMetadata;
use Mago\Sdk\Analyzer\Metadata\TemplateMetadata;
use Mago\Sdk\Analyzer\Metadata\TypeMetadata;
use Mago\Sdk\Analyzer\Metadata\VersionRange;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\GenericParent;
use Mago\Sdk\Analyzer\Type\GenericParentKind;
use Mago\Sdk\Analyzer\Type\Variance;
use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\PHPVersion;
use Mago\Sdk\SourceLocation;
use Mago\Sdk\Span;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:halstead
 * @mago-expect lint:too-many-methods
 */
final class MetadataCodec
{
    private const MAXIMUM_MEMBERS = 65_536;

    public static function readClassLike(PayloadReader $reader): ClassLikeMetadata
    {
        return new ClassLikeMetadata(
            $reader->readBytes(),
            $reader->readBytes(),
            self::readClassLikeKind($reader),
            self::readLocation($reader),
            self::readOptionalLocation($reader),
            new MetadataFlags($reader->readU64()),
            $reader->readOptionalString(),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readStrings($reader),
            self::readOptionalStrings($reader),
            self::readOptionalStrings($reader),
            self::readTemplates($reader),
            self::readAttributes($reader),
            self::readTypeMetadataMap($reader),
            self::readTypes($reader),
            self::readOptionalType($reader),
            self::readOptionalBoolean($reader),
            self::readOptionalBoolean($reader),
            self::readVersionRanges($reader),
        );
    }

    public static function readFunctionLike(PayloadReader $reader): FunctionLikeMetadata
    {
        $kind = match ($value = $reader->readU8()) {
            1 => FunctionLikeKind::Function_,
            2 => FunctionLikeKind::Method,
            3 => FunctionLikeKind::Closure,
            4 => FunctionLikeKind::ArrowFunction,
            default => throw new ProtocolException("Unknown function-like metadata kind {$value}."),
        };

        $name = $reader->readBytes();
        $originalName = $reader->readBytes();
        $location = self::readLocation($reader);
        $nameLocation = self::readOptionalLocation($reader);
        $parameters = [];
        $parameterCount = $reader->readCount(self::MAXIMUM_MEMBERS);
        for ($index = 0; $index < $parameterCount; ++$index) {
            $parameters[] = self::readParameter($reader);
        }

        $declaredReturnType = self::readOptionalTypeMetadata($reader);
        $returnType = self::readOptionalTypeMetadata($reader);
        $templates = self::readTemplates($reader);
        $attributes = self::readAttributes($reader);
        $thrownTypes = [];
        $thrownCount = $reader->readCount(self::MAXIMUM_MEMBERS);
        for ($index = 0; $index < $thrownCount; ++$index) {
            $thrownTypes[] = self::readTypeMetadata($reader);
        }

        $globals = self::readStrings($reader);
        $hasDocblock = $reader->readBoolean();
        $flags = new MetadataFlags($reader->readU64());
        $availableVersions = self::readVersionRanges($reader);
        $visibility = null;
        $final = false;
        $abstract = false;
        $static = false;
        $constructor = false;
        $whereConstraints = [];
        if ($reader->readBoolean()) {
            $visibility = self::readVisibility($reader);
            $final = $reader->readBoolean();
            $abstract = $reader->readBoolean();
            $static = $reader->readBoolean();
            $constructor = $reader->readBoolean();
            $count = $reader->readCount(self::MAXIMUM_MEMBERS);
            for ($index = 0; $index < $count; ++$index) {
                $whereConstraints[$reader->readBytes()] = self::readTypeMetadata($reader);
            }
        }

        return new FunctionLikeMetadata(
            $kind,
            $name,
            $originalName,
            $location,
            $nameLocation,
            $parameters,
            $declaredReturnType,
            $returnType,
            $templates,
            $attributes,
            $thrownTypes,
            $globals,
            $hasDocblock,
            $flags,
            $availableVersions,
            $visibility,
            $final,
            $abstract,
            $static,
            $constructor,
            $whereConstraints,
        );
    }

    public static function readProperty(PayloadReader $reader): PropertyMetadata
    {
        return new PropertyMetadata(
            $reader->readBytes(),
            self::readOptionalLocation($reader),
            self::readOptionalLocation($reader),
            self::readVisibility($reader),
            self::readVisibility($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalTypeMetadata($reader),
            new MetadataFlags($reader->readU64()),
            self::readPropertyHooks($reader),
            self::readVersionRanges($reader),
        );
    }

    /** @return array<string, PropertyHookMetadata> */
    private static function readPropertyHooks(PayloadReader $reader): array
    {
        $hooks = [];
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        for ($index = 0; $index < $count; ++$index) {
            $name = $reader->readBytes();
            $hooks[$name] = new PropertyHookMetadata(
                $name,
                self::readLocation($reader),
                new MetadataFlags($reader->readU64()),
                $reader->readBoolean() ? self::readParameter($reader) : null,
                $reader->readBoolean(),
                $reader->readBoolean(),
                self::readAttributes($reader),
                self::readOptionalTypeMetadata($reader),
                $reader->readBoolean(),
            );
        }

        return $hooks;
    }

    public static function readClassConstant(PayloadReader $reader): ClassConstantMetadata
    {
        return new ClassConstantMetadata(
            $reader->readBytes(),
            self::readLocation($reader),
            self::readVisibility($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalType($reader),
            self::readAttributes($reader),
            new MetadataFlags($reader->readU64()),
            self::readVersionRanges($reader),
        );
    }

    public static function readEnumCase(PayloadReader $reader): EnumCaseMetadata
    {
        return new EnumCaseMetadata(
            $reader->readBytes(),
            self::readLocation($reader),
            self::readLocation($reader),
            self::readOptionalType($reader),
            self::readAttributes($reader),
            new MetadataFlags($reader->readU64()),
            self::readVersionRanges($reader),
        );
    }

    public static function readConstant(PayloadReader $reader): ConstantMetadata
    {
        return new ConstantMetadata(
            $reader->readBytes(),
            self::readLocation($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalType($reader),
            self::readAttributes($reader),
            new MetadataFlags($reader->readU64()),
            self::readVersionRanges($reader),
        );
    }

    private static function readParameter(PayloadReader $reader): ParameterMetadata
    {
        return new ParameterMetadata(
            $reader->readBytes(),
            self::readLocation($reader),
            self::readLocation($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalTypeMetadata($reader),
            self::readOptionalTypeMetadata($reader),
            self::readAttributes($reader),
            new MetadataFlags($reader->readU64()),
        );
    }

    private static function readTypeMetadata(PayloadReader $reader): TypeMetadata
    {
        return new TypeMetadata(
            self::readLocation($reader),
            TypeCodec::readComplete($reader),
            $reader->readBoolean(),
            $reader->readBoolean(),
        );
    }

    private static function readOptionalTypeMetadata(PayloadReader $reader): ?TypeMetadata
    {
        return $reader->readBoolean() ? self::readTypeMetadata($reader) : null;
    }

    private static function readOptionalType(PayloadReader $reader): ?Type
    {
        return $reader->readBoolean() ? TypeCodec::readComplete($reader) : null;
    }

    /** @return list<Type> */
    private static function readTypes(PayloadReader $reader): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $types = [];
        for ($index = 0; $index < $count; ++$index) {
            $types[] = TypeCodec::readComplete($reader);
        }

        return $types;
    }

    /** @return list<TemplateMetadata> */
    private static function readTemplates(PayloadReader $reader): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $templates = [];
        for ($index = 0; $index < $count; ++$index) {
            $templates[] = new TemplateMetadata(
                $reader->readBytes(),
                self::readGenericParent($reader),
                TypeCodec::readComplete($reader),
                self::readOptionalType($reader),
                self::readVariance($reader),
                $reader->readBoolean(),
            );
        }

        return $templates;
    }

    /** @return list<AttributeMetadata> */
    private static function readAttributes(PayloadReader $reader): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $attributes = [];
        for ($index = 0; $index < $count; ++$index) {
            $attributes[] = new AttributeMetadata($reader->readBytes(), self::readLocation($reader));
        }

        return $attributes;
    }

    /** @return array<string, TypeMetadata> */
    private static function readTypeMetadataMap(PayloadReader $reader): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $values = [];
        for ($index = 0; $index < $count; ++$index) {
            $values[$reader->readBytes()] = self::readTypeMetadata($reader);
        }

        return $values;
    }

    private static function readLocation(PayloadReader $reader): SourceLocation
    {
        $file = $reader->readBoolean() ? $reader->readBytes() : null;

        return new SourceLocation($file, new Span($reader->readU32(), $reader->readU32()));
    }

    private static function readOptionalLocation(PayloadReader $reader): ?SourceLocation
    {
        return $reader->readBoolean() ? self::readLocation($reader) : null;
    }

    /** @return list<string> */
    private static function readStrings(PayloadReader $reader): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $values = [];
        for ($index = 0; $index < $count; ++$index) {
            $values[] = $reader->readBytes();
        }

        return $values;
    }

    /** @return list<string>|null */
    private static function readOptionalStrings(PayloadReader $reader): ?array
    {
        return $reader->readBoolean() ? self::readStrings($reader) : null;
    }

    /** @return list<VersionRange> */
    private static function readVersionRanges(PayloadReader $reader): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $ranges = [];
        for ($index = 0; $index < $count; ++$index) {
            $minimum = $reader->readBoolean() ? new PHPVersion($reader->readU32()) : null;
            $maximum = $reader->readBoolean() ? new PHPVersion($reader->readU32()) : null;
            $ranges[] = new VersionRange($minimum, $maximum);
        }

        return $ranges;
    }

    private static function readOptionalBoolean(PayloadReader $reader): ?bool
    {
        return match ($value = $reader->readU8()) {
            0 => null,
            1 => false,
            2 => true,
            default => throw new ProtocolException("Unknown optional boolean value {$value}."),
        };
    }

    private static function readClassLikeKind(PayloadReader $reader): ClassLikeKind
    {
        return match ($value = $reader->readU8()) {
            1 => ClassLikeKind::Class_,
            2 => ClassLikeKind::Enum,
            3 => ClassLikeKind::Trait,
            4 => ClassLikeKind::Interface,
            default => throw new ProtocolException("Unknown class-like metadata kind {$value}."),
        };
    }

    private static function readVisibility(PayloadReader $reader): Visibility
    {
        return match ($value = $reader->readU8()) {
            1 => Visibility::Public,
            2 => Visibility::Protected,
            3 => Visibility::Private,
            default => throw new ProtocolException("Unknown metadata visibility {$value}."),
        };
    }

    private static function readVariance(PayloadReader $reader): Variance
    {
        return match ($value = $reader->readU8()) {
            1 => Variance::Invariant,
            2 => Variance::Covariant,
            3 => Variance::Contravariant,
            4 => Variance::Bivariant,
            default => throw new ProtocolException("Unknown metadata variance {$value}."),
        };
    }

    private static function readGenericParent(PayloadReader $reader): GenericParent
    {
        return match ($value = $reader->readU8()) {
            1 => new GenericParent(GenericParentKind::ClassLike, $reader->readBytes()),
            2 => new GenericParent(GenericParentKind::FunctionLike, $reader->readBytes(), $reader->readBytes()),
            default => throw new ProtocolException("Unknown metadata generic parent {$value}."),
        };
    }
}
