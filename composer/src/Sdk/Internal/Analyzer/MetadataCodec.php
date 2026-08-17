<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Assertion\ArrayKeyAssertion;
use Mago\Sdk\Analyzer\Assertion\ArrayKeyAssertionKind;
use Mago\Sdk\Analyzer\Assertion\Assertion;
use Mago\Sdk\Analyzer\Assertion\CountabilityAssertion;
use Mago\Sdk\Analyzer\Assertion\CountabilityAssertionKind;
use Mago\Sdk\Analyzer\Assertion\IntegerAssertion;
use Mago\Sdk\Analyzer\Assertion\IntegerAssertionKind;
use Mago\Sdk\Analyzer\Assertion\SimpleAssertion;
use Mago\Sdk\Analyzer\Assertion\SimpleAssertionKind;
use Mago\Sdk\Analyzer\Assertion\TypeAssertion;
use Mago\Sdk\Analyzer\Assertion\TypeAssertionKind;
use Mago\Sdk\Analyzer\Assertion\VariableAssertion;
use Mago\Sdk\Analyzer\Assertion\VariableAssertionKind;
use Mago\Sdk\Analyzer\Metadata\AttributeArgumentMetadata;
use Mago\Sdk\Analyzer\Metadata\AttributeMetadata;
use Mago\Sdk\Analyzer\Metadata\ClassConstantMetadata;
use Mago\Sdk\Analyzer\Metadata\ClassLikeKind;
use Mago\Sdk\Analyzer\Metadata\ClassLikeMetadata;
use Mago\Sdk\Analyzer\Metadata\ConstantMetadata;
use Mago\Sdk\Analyzer\Metadata\EnumCaseMetadata;
use Mago\Sdk\Analyzer\Metadata\FunctionLikeKind;
use Mago\Sdk\Analyzer\Metadata\FunctionLikeMetadata;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\Metadata\MetadataFlags;
use Mago\Sdk\Analyzer\Metadata\MethodFields;
use Mago\Sdk\Analyzer\Metadata\MethodMetadataProjection;
use Mago\Sdk\Analyzer\Metadata\ParameterMetadata;
use Mago\Sdk\Analyzer\Metadata\PropertyHookMetadata;
use Mago\Sdk\Analyzer\Metadata\PropertyMetadata;
use Mago\Sdk\Analyzer\Metadata\TemplateMetadata;
use Mago\Sdk\Analyzer\Metadata\TypeMetadata;
use Mago\Sdk\Analyzer\Metadata\VersionRange;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\ArrayKey;
use Mago\Sdk\Analyzer\Type\ArrayKeyKind;
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
 * @mago-expect lint:kan-defect
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
        $identifier = TypeCodec::readFunctionLikeIdentifier($reader);
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
        $assertions = self::readAssertions($reader);
        $ifTrueAssertions = self::readAssertions($reader);
        $ifFalseAssertions = self::readAssertions($reader);
        $assertionsInferred = $reader->readBoolean();
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
            $identifier,
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
            $assertions,
            $ifTrueAssertions,
            $ifFalseAssertions,
            $assertionsInferred,
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

    public static function readMethodProjection(PayloadReader $reader, int $fields): MethodMetadataProjection
    {
        $method = new MemberIdentifier($reader->readBytes(), $reader->readBytes());
        $identifier = TypeCodec::readFunctionLikeIdentifier($reader);
        $name = null;
        $originalName = null;
        if (($fields & MethodFields::NAMES) !== 0) {
            $name = $reader->readBytes();
            $originalName = $reader->readBytes();
        }

        $location = null;
        $nameLocation = null;
        if (($fields & MethodFields::LOCATIONS) !== 0) {
            $location = self::readLocation($reader);
            $nameLocation = self::readOptionalLocation($reader);
        }

        $parameters = null;
        if (($fields & MethodFields::PARAMETERS) !== 0) {
            $parameters = [];
            $count = $reader->readCount(self::MAXIMUM_MEMBERS);
            for ($index = 0; $index < $count; ++$index) {
                $parameters[] = self::readParameter($reader);
            }
        }

        $declaredReturnType = null;
        $returnType = null;
        if (($fields & MethodFields::RETURN_TYPES) !== 0) {
            $declaredReturnType = self::readOptionalTypeMetadata($reader);
            $returnType = self::readOptionalTypeMetadata($reader);
        }

        $templates = null;
        if (($fields & MethodFields::TEMPLATES) !== 0) {
            $templates = self::readTemplates($reader);
        }

        $attributes = null;
        if (($fields & MethodFields::ATTRIBUTES) !== 0) {
            $attributes = self::readAttributes($reader);
        }

        $thrownTypes = null;
        if (($fields & MethodFields::THROWN_TYPES) !== 0) {
            $thrownTypes = [];
            $count = $reader->readCount(self::MAXIMUM_MEMBERS);
            for ($index = 0; $index < $count; ++$index) {
                $thrownTypes[] = self::readTypeMetadata($reader);
            }
        }

        $assertions = null;
        $ifTrueAssertions = null;
        $ifFalseAssertions = null;
        $assertionsInferred = null;
        if (($fields & MethodFields::ASSERTIONS) !== 0) {
            $assertions = self::readAssertions($reader);
            $ifTrueAssertions = self::readAssertions($reader);
            $ifFalseAssertions = self::readAssertions($reader);
            $assertionsInferred = $reader->readBoolean();
        }

        $globalsAccessed = ($fields & MethodFields::GLOBALS) !== 0 ? self::readStrings($reader) : null;
        $hasDocblock = ($fields & MethodFields::DOCBLOCK) !== 0 ? $reader->readBoolean() : null;
        $flags = ($fields & MethodFields::FLAGS) !== 0 ? new MetadataFlags($reader->readU64()) : null;
        $availableVersions = ($fields & MethodFields::AVAILABLE_VERSIONS) !== 0
            ? self::readVersionRanges($reader)
            : null;

        $visibility = null;
        $final = null;
        $abstract = null;
        $static = null;
        $constructor = null;
        if (($fields & MethodFields::METHOD_DETAILS) !== 0) {
            $visibility = self::readVisibility($reader);
            $final = $reader->readBoolean();
            $abstract = $reader->readBoolean();
            $static = $reader->readBoolean();
            $constructor = $reader->readBoolean();
        }

        $whereConstraints = null;
        if (($fields & MethodFields::WHERE_CONSTRAINTS) !== 0) {
            $whereConstraints = [];
            $count = $reader->readCount(self::MAXIMUM_MEMBERS);
            for ($index = 0; $index < $count; ++$index) {
                $whereConstraints[$reader->readBytes()] = self::readTypeMetadata($reader);
            }
        }

        return new MethodMetadataProjection(
            method: $method,
            identifier: $identifier,
            fields: $fields,
            name: $name,
            originalName: $originalName,
            location: $location,
            nameLocation: $nameLocation,
            parameters: $parameters,
            declaredReturnType: $declaredReturnType,
            returnType: $returnType,
            templates: $templates,
            attributes: $attributes,
            thrownTypes: $thrownTypes,
            assertions: $assertions,
            ifTrueAssertions: $ifTrueAssertions,
            ifFalseAssertions: $ifFalseAssertions,
            assertionsInferred: $assertionsInferred,
            globalsAccessed: $globalsAccessed,
            hasDocblock: $hasDocblock,
            flags: $flags,
            availableVersions: $availableVersions,
            visibility: $visibility,
            final: $final,
            abstract: $abstract,
            static: $static,
            constructor: $constructor,
            whereConstraints: $whereConstraints,
        );
    }

    /** @return array<string, list<Assertion>> */
    private static function readAssertions(PayloadReader $reader): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $assertions = [];
        for ($index = 0; $index < $count; ++$index) {
            $variable = $reader->readBytes();
            $assertionCount = $reader->readCount(self::MAXIMUM_MEMBERS);
            $values = [];
            for ($assertionIndex = 0; $assertionIndex < $assertionCount; ++$assertionIndex) {
                $values[] = self::readAssertion($reader);
            }
            $assertions[$variable] = $values;
        }

        return $assertions;
    }

    private static function readAssertion(PayloadReader $reader): Assertion
    {
        return match ($kind = $reader->readU8()) {
            1 => new SimpleAssertion(SimpleAssertionKind::Any),
            2 => new TypeAssertion(TypeAssertionKind::IsType, TypeCodec::readComplete($reader)),
            3 => new TypeAssertion(TypeAssertionKind::IsNotType, TypeCodec::readComplete($reader)),
            4 => new SimpleAssertion(SimpleAssertionKind::Falsy),
            5 => new SimpleAssertion(SimpleAssertionKind::Truthy),
            6 => new TypeAssertion(TypeAssertionKind::IsIdentical, TypeCodec::readComplete($reader)),
            7 => new TypeAssertion(TypeAssertionKind::IsNotIdentical, TypeCodec::readComplete($reader)),
            8 => new TypeAssertion(TypeAssertionKind::IsEqual, TypeCodec::readComplete($reader)),
            9 => new TypeAssertion(TypeAssertionKind::IsNotEqual, TypeCodec::readComplete($reader)),
            10 => new SimpleAssertion(SimpleAssertionKind::IsEqualIsset),
            11 => new SimpleAssertion(SimpleAssertionKind::IsIsset),
            12 => new SimpleAssertion(SimpleAssertionKind::IsNotIsset),
            13 => new SimpleAssertion(SimpleAssertionKind::HasStringArrayAccess),
            14 => new SimpleAssertion(SimpleAssertionKind::HasIntOrStringArrayAccess),
            15 => new SimpleAssertion(SimpleAssertionKind::ArrayKeyExists),
            16 => new SimpleAssertion(SimpleAssertionKind::ArrayKeyDoesNotExist),
            17 => new TypeAssertion(TypeAssertionKind::InArray, TypeCodec::readComplete($reader)),
            18 => new TypeAssertion(TypeAssertionKind::NotInArray, TypeCodec::readComplete($reader)),
            19 => new ArrayKeyAssertion(ArrayKeyAssertionKind::HasKey, self::readArrayKey($reader)),
            20 => new ArrayKeyAssertion(ArrayKeyAssertionKind::DoesNotHaveKey, self::readArrayKey($reader)),
            21 => new ArrayKeyAssertion(ArrayKeyAssertionKind::HasNonnullEntryForKey, self::readArrayKey($reader)),
            22 => new ArrayKeyAssertion(
                ArrayKeyAssertionKind::DoesNotHaveNonnullEntryForKey,
                self::readArrayKey($reader),
            ),
            23 => new SimpleAssertion(SimpleAssertionKind::Empty),
            24 => new SimpleAssertion(SimpleAssertionKind::NonEmpty),
            25 => new CountabilityAssertion(CountabilityAssertionKind::NonEmpty, $reader->readBoolean()),
            26 => new SimpleAssertion(SimpleAssertionKind::EmptyCountable),
            27 => new IntegerAssertion(IntegerAssertionKind::HasExactCount, $reader->readU64()),
            28 => new IntegerAssertion(IntegerAssertionKind::HasAtLeastCount, $reader->readU64()),
            29 => new IntegerAssertion(IntegerAssertionKind::DoesNotHaveExactCount, $reader->readU64()),
            30 => new IntegerAssertion(IntegerAssertionKind::DoesNotHaveAtLeastCount, $reader->readU64()),
            31 => new IntegerAssertion(IntegerAssertionKind::IsLessThan, $reader->readI64()),
            32 => new IntegerAssertion(IntegerAssertionKind::IsLessThanOrEqual, $reader->readI64()),
            33 => new IntegerAssertion(IntegerAssertionKind::IsGreaterThan, $reader->readI64()),
            34 => new IntegerAssertion(IntegerAssertionKind::IsGreaterThanOrEqual, $reader->readI64()),
            35 => new IntegerAssertion(IntegerAssertionKind::IsLessThanFromBound, $reader->readI64()),
            36 => new IntegerAssertion(IntegerAssertionKind::IsLessThanOrEqualFromBound, $reader->readI64()),
            37 => new IntegerAssertion(IntegerAssertionKind::IsGreaterThanFromBound, $reader->readI64()),
            38 => new IntegerAssertion(IntegerAssertionKind::IsGreaterThanOrEqualFromBound, $reader->readI64()),
            39 => new VariableAssertion(VariableAssertionKind::IsLessThan, $reader->readBytes()),
            40 => new VariableAssertion(VariableAssertionKind::IsLessThanOrEqual, $reader->readBytes()),
            41 => new VariableAssertion(VariableAssertionKind::IsGreaterThan, $reader->readBytes()),
            42 => new VariableAssertion(VariableAssertionKind::IsGreaterThanOrEqual, $reader->readBytes()),
            43 => new SimpleAssertion(SimpleAssertionKind::Countable),
            44 => new CountabilityAssertion(CountabilityAssertionKind::NotCountable, $reader->readBoolean()),
            default => throw new ProtocolException("Unknown function-like assertion kind {$kind}."),
        };
    }

    private static function readArrayKey(PayloadReader $reader): ArrayKey
    {
        return match ($kind = $reader->readU8()) {
            1 => new ArrayKey(ArrayKeyKind::Integer, $reader->readI64()),
            2 => new ArrayKey(ArrayKeyKind::String, $reader->readBytes()),
            3 => new ArrayKey(ArrayKeyKind::ClassLikeConstant, $reader->readBytes(), $reader->readBytes()),
            default => throw new ProtocolException("Unknown assertion array-key kind {$kind}."),
        };
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
            self::readAttributes($reader),
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
            $name = $reader->readBytes();
            $location = self::readLocation($reader);
            $arguments = [];
            $argumentCount = $reader->readCount(self::MAXIMUM_MEMBERS);
            for ($argumentIndex = 0; $argumentIndex < $argumentCount; ++$argumentIndex) {
                $arguments[] = new AttributeArgumentMetadata(
                    $reader->readOptionalString(),
                    self::readLocation($reader),
                    self::readOptionalLocation($reader),
                    self::readOptionalLocation($reader),
                    self::readOptionalType($reader),
                );
            }
            $attributes[] = new AttributeMetadata($name, $location, $arguments);
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
