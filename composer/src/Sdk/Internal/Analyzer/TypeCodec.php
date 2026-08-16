<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Analyzer;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\AliasType;
use Mago\Sdk\Analyzer\Type\AnyObjectType;
use Mago\Sdk\Analyzer\Type\ArrayItem;
use Mago\Sdk\Analyzer\Type\ArrayKey;
use Mago\Sdk\Analyzer\Type\ArrayKeyKind;
use Mago\Sdk\Analyzer\Type\AtomicType;
use Mago\Sdk\Analyzer\Type\CallableConstraint;
use Mago\Sdk\Analyzer\Type\CallableParameter;
use Mago\Sdk\Analyzer\Type\CallableSignature;
use Mago\Sdk\Analyzer\Type\CallableType;
use Mago\Sdk\Analyzer\Type\ClassLikeStringKind;
use Mago\Sdk\Analyzer\Type\ClassLikeStringType;
use Mago\Sdk\Analyzer\Type\ClassLikeStringVariant;
use Mago\Sdk\Analyzer\Type\ConditionalType;
use Mago\Sdk\Analyzer\Type\DerivedType;
use Mago\Sdk\Analyzer\Type\DerivedTypeKind;
use Mago\Sdk\Analyzer\Type\EnumType;
use Mago\Sdk\Analyzer\Type\FloatType;
use Mago\Sdk\Analyzer\Type\FloatTypeKind;
use Mago\Sdk\Analyzer\Type\FunctionLikeIdentifier;
use Mago\Sdk\Analyzer\Type\FunctionLikeKind;
use Mago\Sdk\Analyzer\Type\GenericParameterType;
use Mago\Sdk\Analyzer\Type\GenericParent;
use Mago\Sdk\Analyzer\Type\GenericParentKind;
use Mago\Sdk\Analyzer\Type\IntegerType;
use Mago\Sdk\Analyzer\Type\IntegerTypeKind;
use Mago\Sdk\Analyzer\Type\IterableType;
use Mago\Sdk\Analyzer\Type\KeyedArrayType;
use Mago\Sdk\Analyzer\Type\ListElement;
use Mago\Sdk\Analyzer\Type\ListType;
use Mago\Sdk\Analyzer\Type\MixedTruthiness;
use Mago\Sdk\Analyzer\Type\MixedType;
use Mago\Sdk\Analyzer\Type\NamedObjectType;
use Mago\Sdk\Analyzer\Type\ObjectProperty;
use Mago\Sdk\Analyzer\Type\ObjectShapeType;
use Mago\Sdk\Analyzer\Type\ObjectWithMethodType;
use Mago\Sdk\Analyzer\Type\ObjectWithPropertyType;
use Mago\Sdk\Analyzer\Type\ReferenceSelectorKind;
use Mago\Sdk\Analyzer\Type\ReferenceType;
use Mago\Sdk\Analyzer\Type\ReferenceTypeKind;
use Mago\Sdk\Analyzer\Type\ResourceType;
use Mago\Sdk\Analyzer\Type\ScalarType;
use Mago\Sdk\Analyzer\Type\ScalarTypeKind;
use Mago\Sdk\Analyzer\Type\SimpleAtomicType;
use Mago\Sdk\Analyzer\Type\SimpleAtomicTypeKind;
use Mago\Sdk\Analyzer\Type\StringCasing;
use Mago\Sdk\Analyzer\Type\StringLiteralKind;
use Mago\Sdk\Analyzer\Type\StringType;
use Mago\Sdk\Analyzer\Type\TypeFlags;
use Mago\Sdk\Analyzer\Type\VariableType;
use Mago\Sdk\Analyzer\Type\Variance;
use Mago\Sdk\Analyzer\Type\Visibility;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;

use function array_slice;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 * @mago-expect lint:no-boolean-flag-parameter
 * @mago-expect lint:too-many-methods
 */
final class TypeCodec
{
    private const MAXIMUM_MEMBERS = 65_536;

    public static function read(PayloadReader $reader, ?string $description = null): Type
    {
        [$handle, $flags, $atomicTypes] = self::readUnion($reader, true);

        return Type::reference($handle, $description ?? self::describeAtomics($atomicTypes), $atomicTypes, $flags);
    }

    public static function readComplete(PayloadReader $reader): Type
    {
        [, $flags, $atomicTypes] = self::readUnion($reader, false);
        $first = $atomicTypes[0];
        $type = Type::fromAtomics($first, ...array_slice($atomicTypes, 1));

        return $type->withFlags($flags);
    }

    public static function encode(Type $_type): string
    {
        $writer = new PayloadWriter();
        self::writeUnion($writer, $_type);

        return $writer->finish();
    }

    public static function writeComplete(PayloadWriter $writer, Type $type): void
    {
        self::writeUnion($writer, $type);
    }

    private static function writeUnion(PayloadWriter $writer, Type $type): void
    {
        $flags = $type->flags;
        $bits = 0;
        $bits |= (int) $flags->hadTemplate;
        $bits |= (int) $flags->byReference << 1;
        $bits |= (int) $flags->referenceFree << 2;
        $bits |= (int) $flags->possiblyUndefinedFromTry << 3;
        $bits |= (int) $flags->possiblyUndefined << 4;
        $bits |= (int) $flags->ignoreNullableIssues << 5;
        $bits |= (int) $flags->ignoreFalsableIssues << 6;
        $bits |= (int) $flags->fromTemplateDefault << 7;
        $bits |= (int) $flags->populated << 8;
        $bits |= (int) $flags->nullsafeNull << 9;
        $bits |= (int) $flags->fromUnspecifiedTemplate << 10;
        $writer->writeU16($bits);
        $writer->writeCount($type->atomicTypes);
        foreach ($type->atomicTypes as $atomicType) {
            self::writeAtomic($writer, $atomicType);
        }
    }

    private static function writeAtomic(PayloadWriter $writer, AtomicType $atomic): void
    {
        if ($atomic instanceof ScalarType) {
            $writer->writeU8(1);
            self::writeScalar($writer, $atomic);
            return;
        }
        if ($atomic instanceof CallableType) {
            $writer->writeU8(2);
            self::writeCallable($writer, $atomic);
            return;
        }
        if ($atomic instanceof MixedType) {
            $writer->writeU8(3);
            self::writeMixed($writer, $atomic);
            return;
        }
        if (
            $atomic instanceof AnyObjectType
            || $atomic instanceof NamedObjectType
            || $atomic instanceof EnumType
            || $atomic instanceof ObjectShapeType
            || $atomic instanceof ObjectWithMethodType
            || $atomic instanceof ObjectWithPropertyType
        ) {
            $writer->writeU8(4);
            self::writeObject($writer, $atomic);
            return;
        }
        if ($atomic instanceof ListType || $atomic instanceof KeyedArrayType) {
            $writer->writeU8(5);
            self::writeArray($writer, $atomic);
            return;
        }
        if ($atomic instanceof IterableType) {
            $writer->writeU8(6);
            self::writeUnion($writer, $atomic->keyType);
            self::writeUnion($writer, $atomic->valueType);
            self::writeOptionalAtomics($writer, $atomic->intersections);
            return;
        }
        if ($atomic instanceof ResourceType) {
            $writer->writeU8(7);
            $writer->writeU8(match ($atomic->closed) {
                null => 0,
                false => 1,
                true => 2,
            });
            return;
        }
        if ($atomic instanceof ReferenceType) {
            $writer->writeU8(8);
            self::writeReference($writer, $atomic);
            return;
        }
        if ($atomic instanceof GenericParameterType) {
            $writer->writeU8(9);
            $writer->writeBytes($atomic->name);
            self::writeUnion($writer, $atomic->constraint);
            self::writeGenericParent($writer, $atomic->definingEntity);
            self::writeOptionalAtomics($writer, $atomic->intersections);
            return;
        }
        if ($atomic instanceof VariableType) {
            $writer->writeU8(10);
            $writer->writeBytes($atomic->name);
            return;
        }
        if ($atomic instanceof ConditionalType) {
            $writer->writeU8(11);
            self::writeUnion($writer, $atomic->subject);
            self::writeUnion($writer, $atomic->target);
            self::writeUnion($writer, $atomic->then);
            self::writeUnion($writer, $atomic->otherwise);
            $writer->writeBoolean($atomic->negated);
            return;
        }
        if ($atomic instanceof DerivedType) {
            $writer->writeU8(12);
            self::writeDerived($writer, $atomic);
            return;
        }
        if ($atomic instanceof AliasType) {
            $writer->writeU8(13);
            $writer->writeBytes($atomic->class);
            $writer->writeBytes($atomic->alias);
            return;
        }
        if ($atomic instanceof SimpleAtomicType) {
            $writer->writeU8(match ($atomic->kind) {
                SimpleAtomicTypeKind::Never => 14,
                SimpleAtomicTypeKind::Null => 15,
                SimpleAtomicTypeKind::Void => 16,
                SimpleAtomicTypeKind::Placeholder => 17,
            });
            return;
        }

        throw new ProtocolException('Cannot encode an unknown analyzer atomic type.');
    }

    private static function writeScalar(PayloadWriter $writer, ScalarType $scalar): void
    {
        $writer->writeU8(match ($scalar->kind) {
            ScalarTypeKind::Scalar => 1,
            ScalarTypeKind::Numeric => 2,
            ScalarTypeKind::ArrayKey => 3,
            ScalarTypeKind::Boolean => 4,
            ScalarTypeKind::Integer => 5,
            ScalarTypeKind::Float => 6,
            ScalarTypeKind::String => 7,
            ScalarTypeKind::ClassLikeString => 8,
        });

        match ($scalar->kind) {
            ScalarTypeKind::Boolean => $writer->writeU8(match ($scalar->refinement) {
                null => 0,
                false => 1,
                true => 2,
                default => throw new ProtocolException('A boolean scalar has non-boolean refinement data.'),
            }),
            ScalarTypeKind::Integer => self::writeInteger($writer, $scalar->refinement),
            ScalarTypeKind::Float => self::writeFloat($writer, $scalar->refinement),
            ScalarTypeKind::String => self::writeString($writer, $scalar->refinement),
            ScalarTypeKind::ClassLikeString => self::writeClassLikeString($writer, $scalar->refinement),
            default => null,
        };
    }

    private static function writeInteger(
        PayloadWriter $writer,
        bool|IntegerType|FloatType|StringType|ClassLikeStringType|null $type,
    ): void {
        if (!$type instanceof IntegerType) {
            $type = new IntegerType(IntegerTypeKind::General);
        }

        $writer->writeU8(match ($type->kind) {
            IntegerTypeKind::Literal => 1,
            IntegerTypeKind::From => 2,
            IntegerTypeKind::To => 3,
            IntegerTypeKind::Range => 4,
            IntegerTypeKind::General => 5,
            IntegerTypeKind::UnspecifiedLiteral => 6,
        });

        match ($type->kind) {
            IntegerTypeKind::Literal, IntegerTypeKind::From => $writer->writeI64($type->minimum ?? 0),
            IntegerTypeKind::To => $writer->writeI64($type->maximum ?? 0),
            IntegerTypeKind::Range => self::writeIntegerRange($writer, $type),
            default => null,
        };
    }

    private static function writeIntegerRange(PayloadWriter $writer, IntegerType $type): void
    {
        $writer->writeI64($type->minimum ?? 0);
        $writer->writeI64($type->maximum ?? 0);
    }

    private static function writeFloat(
        PayloadWriter $writer,
        bool|IntegerType|FloatType|StringType|ClassLikeStringType|null $type,
    ): void {
        if (!$type instanceof FloatType) {
            $writer->writeU8(1);
            return;
        }

        $writer->writeU8(match ($type->kind) {
            FloatTypeKind::General => 1,
            FloatTypeKind::UnspecifiedLiteral => 2,
            FloatTypeKind::Literal => 3,
        });

        if ($type->kind === FloatTypeKind::Literal) {
            $writer->writeF64($type->value ?? 0.0);
        }
    }

    private static function writeString(
        PayloadWriter $writer,
        bool|IntegerType|FloatType|StringType|ClassLikeStringType|null $type,
    ): void {
        if (!$type instanceof StringType) {
            $type = new StringType(
                StringLiteralKind::General,
                null,
                false,
                false,
                false,
                false,
                StringCasing::Unspecified,
            );
        }

        $writer->writeU8(match ($type->literalKind) {
            StringLiteralKind::General => 0,
            StringLiteralKind::Unspecified => 1,
            StringLiteralKind::Value => 2,
        });

        if ($type->literalKind === StringLiteralKind::Value) {
            $writer->writeBytes($type->literalValue ?? '');
        }

        $bits = 0;
        $bits |= (int) $type->numeric;
        $bits |= (int) $type->truthy << 1;
        $bits |= (int) $type->nonEmpty << 2;
        $bits |= (int) $type->callable << 3;
        $writer->writeU8($bits);
        $writer->writeU8(match ($type->casing) {
            StringCasing::Unspecified => 0,
            StringCasing::Lowercase => 1,
            StringCasing::Uppercase => 2,
        });
    }

    private static function writeClassLikeString(
        PayloadWriter $writer,
        bool|IntegerType|FloatType|StringType|ClassLikeStringType|null $type,
    ): void {
        if (!$type instanceof ClassLikeStringType) {
            throw new ProtocolException('A class-like string scalar requires class-like string refinement data.');
        }

        $writer->writeU8(match ($type->variant) {
            ClassLikeStringVariant::Any => 1,
            ClassLikeStringVariant::Generic => 2,
            ClassLikeStringVariant::Literal => 3,
            ClassLikeStringVariant::OfType => 4,
        });

        if ($type->variant === ClassLikeStringVariant::Literal) {
            $writer->writeBytes($type->literal ?? '');
            return;
        }

        self::writeClassLikeStringKind($writer, $type->kind);
        if ($type->variant === ClassLikeStringVariant::Generic) {
            $writer->writeBytes($type->parameterName ?? '');
            self::writeGenericParent(
                $writer,
                $type->definingEntity ?? throw new ProtocolException(
                    'A generic class-like string requires a defining entity.',
                ),
            );
        }

        if ($type->variant !== ClassLikeStringVariant::Any) {
            self::writeAtomic(
                $writer,
                $type->constraint ?? throw new ProtocolException(
                    'A constrained class-like string requires a constraint.',
                ),
            );
        }
    }

    /** @mago-expect lint:halstead */
    private static function writeCallable(PayloadWriter $writer, CallableType $callable): void
    {
        if ($callable->signature === null) {
            $writer->writeU8(2);
            self::writeFunctionLikeIdentifier(
                $writer,
                $callable->alias ?? throw new ProtocolException('A callable alias requires an identifier.'),
            );

            return;
        }

        $signature = $callable->signature;
        $writer->writeU8(1);
        $writer->writeBoolean($signature->pure);
        $writer->writeBoolean($signature->closure);
        $writer->writeCount($signature->parameters);
        foreach ($signature->parameters as $parameter) {
            $writer->writeBoolean($parameter->name !== null);
            if ($parameter->name !== null) {
                $writer->writeBytes($parameter->name);
            }

            $writer->writeBoolean($parameter->type !== null);
            if ($parameter->type !== null) {
                self::writeUnion($writer, $parameter->type);
            }

            $writer->writeBoolean($parameter->closureThisType !== null);
            if ($parameter->closureThisType !== null) {
                self::writeUnion($writer, $parameter->closureThisType);
            }

            $writer->writeBoolean($parameter->byReference);
            $writer->writeBoolean($parameter->variadic);
            $writer->writeBoolean($parameter->hasDefault);
        }

        $writer->writeBoolean($signature->returnType !== null);
        if ($signature->returnType !== null) {
            self::writeUnion($writer, $signature->returnType);
        }

        $writer->writeBoolean($signature->source !== null);
        if ($signature->source !== null) {
            self::writeFunctionLikeIdentifier($writer, $signature->source);
        }

        $writer->writeCount($signature->constraints);
        foreach ($signature->constraints as $constraint) {
            $writer->writeCount($constraint->parameterNames);
            foreach ($constraint->parameterNames as $name) {
                $writer->writeBytes($name);
            }

            self::writeUnion($writer, $constraint->inputType);
            self::writeUnion($writer, $constraint->parameterType);
        }
    }

    private static function writeMixed(PayloadWriter $writer, MixedType $mixed): void
    {
        $bits = 0;
        $bits |= (int) $mixed->issetFromLoop;
        $bits |= (int) $mixed->nonNull << 1;
        $bits |= (int) $mixed->empty << 2;
        $writer->writeU8($bits);
        $writer->writeU8(match ($mixed->truthiness) {
            MixedTruthiness::Undetermined => 0,
            MixedTruthiness::Truthy => 1,
            MixedTruthiness::Falsy => 2,
        });
    }

    private static function writeObject(PayloadWriter $writer, AtomicType $object): void
    {
        if ($object instanceof AnyObjectType) {
            $writer->writeU8(1);
            return;
        }

        if ($object instanceof NamedObjectType) {
            $writer->writeU8(2);
            $writer->writeBytes($object->name);
            self::writeOptionalTypes($writer, $object->parameters);
            self::writeOptionalVariances($writer, $object->variances);
            $writer->writeBoolean($object->static);
            $writer->writeBoolean($object->isThis);
            self::writeOptionalAtomics($writer, $object->intersections);
            $writer->writeBoolean($object->remappedParameters);
            return;
        }

        if ($object instanceof EnumType) {
            $writer->writeU8(3);
            $writer->writeBytes($object->name);
            $writer->writeOptionalString($object->case);
            return;
        }

        if ($object instanceof ObjectShapeType) {
            $writer->writeU8(4);
            $writer->writeBoolean($object->sealed);
            $writer->writeCount($object->properties);
            foreach ($object->properties as $property) {
                $writer->writeBytes($property->name);
                $writer->writeBoolean($property->optional);
                self::writeUnion($writer, $property->type);
            }

            return;
        }

        if ($object instanceof ObjectWithMethodType) {
            $writer->writeU8(5);
            $writer->writeBytes($object->method);
            self::writeOptionalAtomics($writer, $object->intersections);
            return;
        }

        if ($object instanceof ObjectWithPropertyType) {
            $writer->writeU8(6);
            $writer->writeBytes($object->property);
            self::writeOptionalAtomics($writer, $object->intersections);
            return;
        }

        throw new ProtocolException('Cannot encode an unknown object type.');
    }

    /** @mago-expect lint:halstead */
    private static function writeArray(PayloadWriter $writer, ListType|KeyedArrayType $array): void
    {
        if ($array instanceof ListType) {
            $writer->writeU8(1);
            self::writeUnion($writer, $array->elementType);
            $writer->writeBoolean($array->knownElements !== null);
            if ($array->knownElements !== null) {
                $writer->writeCount($array->knownElements);
                foreach ($array->knownElements as $element) {
                    $writer->writeU64($element->index);
                    $writer->writeBoolean($element->optional);
                    self::writeUnion($writer, $element->type);
                }
            }

            $writer->writeBoolean($array->knownCount !== null);
            if ($array->knownCount !== null) {
                $writer->writeU64($array->knownCount);
            }

            $writer->writeBoolean($array->nonEmpty);
            return;
        }

        $writer->writeU8(2);
        $writer->writeBoolean($array->knownItems !== null);
        if ($array->knownItems !== null) {
            $writer->writeCount($array->knownItems);
            foreach ($array->knownItems as $item) {
                self::writeArrayKey($writer, $item->key);
                $writer->writeBoolean($item->optional);
                self::writeUnion($writer, $item->type);
            }
        }

        $keyType = $array->keyType;
        $valueType = $array->valueType;
        $writer->writeBoolean($keyType !== null && $valueType !== null);
        if ($keyType !== null && $valueType !== null) {
            self::writeUnion($writer, $keyType);
            self::writeUnion($writer, $valueType);
        }

        $writer->writeBoolean($array->nonEmpty);
    }

    private static function writeArrayKey(PayloadWriter $writer, ArrayKey $key): void
    {
        $writer->writeU8(match ($key->kind) {
            ArrayKeyKind::Integer => 1,
            ArrayKeyKind::String => 2,
            ArrayKeyKind::ClassLikeConstant => 3,
        });

        match ($key->kind) {
            ArrayKeyKind::Integer => $writer->writeI64((int) $key->value),
            ArrayKeyKind::String => $writer->writeBytes((string) $key->value),
            ArrayKeyKind::ClassLikeConstant => self::writeClassLikeConstantArrayKey($writer, $key),
        };
    }

    private static function writeClassLikeConstantArrayKey(PayloadWriter $writer, ArrayKey $key): void
    {
        $writer->writeBytes((string) $key->value);
        $writer->writeBytes($key->constant ?? '');
    }

    private static function writeReference(PayloadWriter $writer, ReferenceType $reference): void
    {
        $writer->writeU8(match ($reference->kind) {
            ReferenceTypeKind::Symbol => 1,
            ReferenceTypeKind::Member => 2,
            ReferenceTypeKind::Global => 3,
        });

        if ($reference->kind === ReferenceTypeKind::Symbol) {
            $writer->writeBytes($reference->name ?? '');
            self::writeOptionalTypes($writer, $reference->parameters);
            self::writeOptionalVariances($writer, $reference->variances);
            self::writeOptionalAtomics($writer, $reference->intersections);
            return;
        }

        if ($reference->kind === ReferenceTypeKind::Member) {
            $writer->writeBytes($reference->name ?? '');
        }

        self::writeReferenceSelector(
            $writer,
            $reference->selector ?? throw new ProtocolException('A member or global reference requires a selector.'),
            $reference->member,
        );
    }

    private static function writeReferenceSelector(
        PayloadWriter $writer,
        ReferenceSelectorKind $selector,
        ?string $member,
    ): void {
        $writer->writeU8(match ($selector) {
            ReferenceSelectorKind::Wildcard => 1,
            ReferenceSelectorKind::Identifier => 2,
            ReferenceSelectorKind::StartsWith => 3,
            ReferenceSelectorKind::EndsWith => 4,
        });

        if ($selector !== ReferenceSelectorKind::Wildcard) {
            $writer->writeBytes($member ?? '');
        }
    }

    private static function writeDerived(PayloadWriter $writer, DerivedType $derived): void
    {
        $writer->writeU8(match ($derived->kind) {
            DerivedTypeKind::KeyOf => 1,
            DerivedTypeKind::ValueOf => 2,
            DerivedTypeKind::IntMask => 3,
            DerivedTypeKind::IntMaskOf => 4,
            DerivedTypeKind::PropertiesOf => 5,
            DerivedTypeKind::IndexAccess => 6,
            DerivedTypeKind::New_ => 7,
            DerivedTypeKind::TemplateType => 8,
            DerivedTypeKind::Intersection => 9,
        });

        if ($derived->kind === DerivedTypeKind::PropertiesOf) {
            $writer->writeU8(match ($derived->visibility) {
                null => 0,
                Visibility::Public => 1,
                Visibility::Protected => 2,
                Visibility::Private => 3,
            });
        }

        if ($derived->kind === DerivedTypeKind::IntMask) {
            $writer->writeCount($derived->operands);
        }

        foreach ($derived->operands as $operand) {
            self::writeUnion($writer, $operand);
        }

        if ($derived->kind === DerivedTypeKind::Intersection) {
            $writer->writeCount($derived->intersections);
            foreach ($derived->intersections as $intersection) {
                self::writeAtomic($writer, $intersection);
            }
        }
    }

    private static function writeClassLikeStringKind(PayloadWriter $writer, ?ClassLikeStringKind $kind): void
    {
        $writer->writeU8(match ($kind) {
            ClassLikeStringKind::Class_ => 1,
            ClassLikeStringKind::Interface => 2,
            ClassLikeStringKind::Enum => 3,
            ClassLikeStringKind::Trait => 4,
            null => throw new ProtocolException('A non-literal class-like string requires a kind.'),
        });
    }

    private static function writeGenericParent(PayloadWriter $writer, GenericParent $parent): void
    {
        $writer->writeU8(match ($parent->kind) {
            GenericParentKind::ClassLike => 1,
            GenericParentKind::FunctionLike => 2,
        });

        $writer->writeBytes($parent->name);
        if ($parent->kind === GenericParentKind::FunctionLike) {
            $writer->writeBytes($parent->member ?? '');
        }
    }

    public static function writeFunctionLikeIdentifier(PayloadWriter $writer, FunctionLikeIdentifier $identifier): void
    {
        $writer->writeU8(match ($identifier->kind) {
            FunctionLikeKind::Function_ => 1,
            FunctionLikeKind::Method => 2,
            FunctionLikeKind::Closure => 3,
        });

        if ($identifier->kind === FunctionLikeKind::Method) {
            $writer->writeBytes($identifier->class ?? '');
        }

        $writer->writeBytes($identifier->name);
    }

    /** @param null|list<Type> $types */
    private static function writeOptionalTypes(PayloadWriter $writer, ?array $types): void
    {
        $writer->writeBoolean($types !== null);
        if ($types === null) {
            return;
        }

        $writer->writeCount($types);
        foreach ($types as $type) {
            self::writeUnion($writer, $type);
        }
    }

    /** @param null|list<AtomicType> $atomics */
    private static function writeOptionalAtomics(PayloadWriter $writer, ?array $atomics): void
    {
        $writer->writeBoolean($atomics !== null);
        if ($atomics === null) {
            return;
        }

        $writer->writeCount($atomics);
        foreach ($atomics as $atomic) {
            self::writeAtomic($writer, $atomic);
        }
    }

    /** @param null|list<Variance> $variances */
    private static function writeOptionalVariances(PayloadWriter $writer, ?array $variances): void
    {
        $writer->writeBoolean($variances !== null);
        if ($variances === null) {
            return;
        }

        $writer->writeCount($variances);
        foreach ($variances as $variance) {
            $writer->writeU8(match ($variance) {
                Variance::Invariant => 1,
                Variance::Covariant => 2,
                Variance::Contravariant => 3,
                Variance::Bivariant => 4,
            });
        }
    }

    /** @return array{int<0, 4294967295>, TypeFlags, non-empty-list<AtomicType>} */
    private static function readUnion(PayloadReader $reader, bool $references): array
    {
        $handle = $reader->readU32();
        $bits = $reader->readU16();
        $flags = new TypeFlags(
            hadTemplate: ($bits & 1) !== 0,
            byReference: ($bits & (1 << 1)) !== 0,
            referenceFree: ($bits & (1 << 2)) !== 0,
            possiblyUndefinedFromTry: ($bits & (1 << 3)) !== 0,
            possiblyUndefined: ($bits & (1 << 4)) !== 0,
            ignoreNullableIssues: ($bits & (1 << 5)) !== 0,
            ignoreFalsableIssues: ($bits & (1 << 6)) !== 0,
            fromTemplateDefault: ($bits & (1 << 7)) !== 0,
            populated: ($bits & (1 << 8)) !== 0,
            nullsafeNull: ($bits & (1 << 9)) !== 0,
            fromUnspecifiedTemplate: ($bits & (1 << 10)) !== 0,
        );

        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        if ($count === 0) {
            throw new ProtocolException('An analyzer union snapshot contains no atomic types.');
        }

        $atomicTypes = [];
        for ($index = 0; $index < $count; ++$index) {
            $atomicTypes[] = self::readAtomic($reader, $references);
        }

        return [$handle, $flags, $atomicTypes];
    }

    private static function readNestedType(PayloadReader $reader, bool $references): Type
    {
        [$handle, $flags, $atomicTypes] = self::readUnion($reader, $references);

        if (!$references) {
            $first = $atomicTypes[0];
            return Type::fromAtomics($first, ...array_slice($atomicTypes, 1))->withFlags($flags);
        }

        return Type::reference($handle, self::describeAtomics($atomicTypes), $atomicTypes, $flags);
    }

    private static function readAtomic(PayloadReader $reader, bool $references): AtomicType
    {
        return match ($tag = $reader->readU8()) {
            1 => self::readScalar($reader, $references),
            2 => self::readCallable($reader, $references),
            3 => self::readMixed($reader),
            4 => self::readObject($reader, $references),
            5 => self::readArray($reader, $references),
            6 => new IterableType(
                self::readNestedType($reader, $references),
                self::readNestedType($reader, $references),
                self::readOptionalAtomics($reader, $references),
            ),
            7 => new ResourceType(match ($state = $reader->readU8()) {
                0 => null,
                1 => false,
                2 => true,
                default => throw new ProtocolException("Unknown resource state {$state}."),
            }),
            8 => self::readReference($reader, $references),
            9 => new GenericParameterType(
                $reader->readBytes(),
                self::readNestedType($reader, $references),
                self::readGenericParent($reader),
                self::readOptionalAtomics($reader, $references),
            ),
            10 => new VariableType($reader->readBytes()),
            11 => new ConditionalType(
                self::readNestedType($reader, $references),
                self::readNestedType($reader, $references),
                self::readNestedType($reader, $references),
                self::readNestedType($reader, $references),
                $reader->readBoolean(),
            ),
            12 => self::readDerived($reader, $references),
            13 => new AliasType($reader->readBytes(), $reader->readBytes()),
            14 => new SimpleAtomicType(SimpleAtomicTypeKind::Never),
            15 => new SimpleAtomicType(SimpleAtomicTypeKind::Null),
            16 => new SimpleAtomicType(SimpleAtomicTypeKind::Void),
            17 => new SimpleAtomicType(SimpleAtomicTypeKind::Placeholder),
            default => throw new ProtocolException("Unknown analyzer atomic type tag {$tag}."),
        };
    }

    private static function readScalar(PayloadReader $reader, bool $references): ScalarType
    {
        return match ($kind = $reader->readU8()) {
            1 => new ScalarType(ScalarTypeKind::Scalar),
            2 => new ScalarType(ScalarTypeKind::Numeric),
            3 => new ScalarType(ScalarTypeKind::ArrayKey),
            4 => new ScalarType(ScalarTypeKind::Boolean, match ($value = $reader->readU8()) {
                0 => null,
                1 => false,
                2 => true,
                default => throw new ProtocolException("Unknown boolean refinement {$value}."),
            }),
            5 => new ScalarType(ScalarTypeKind::Integer, self::readInteger($reader)),
            6 => new ScalarType(ScalarTypeKind::Float, self::readFloat($reader)),
            7 => new ScalarType(ScalarTypeKind::String, self::readString($reader)),
            8 => new ScalarType(ScalarTypeKind::ClassLikeString, self::readClassLikeString($reader, $references)),
            default => throw new ProtocolException("Unknown analyzer scalar type kind {$kind}."),
        };
    }

    private static function readInteger(PayloadReader $reader): IntegerType
    {
        return match ($kind = $reader->readU8()) {
            1 => new IntegerType(IntegerTypeKind::Literal, $value = $reader->readI64(), $value),
            2 => new IntegerType(IntegerTypeKind::From, $reader->readI64()),
            3 => new IntegerType(IntegerTypeKind::To, null, $reader->readI64()),
            4 => new IntegerType(IntegerTypeKind::Range, $reader->readI64(), $reader->readI64()),
            5 => new IntegerType(IntegerTypeKind::General),
            6 => new IntegerType(IntegerTypeKind::UnspecifiedLiteral),
            default => throw new ProtocolException("Unknown analyzer integer type kind {$kind}."),
        };
    }

    private static function readFloat(PayloadReader $reader): FloatType
    {
        return match ($kind = $reader->readU8()) {
            1 => new FloatType(FloatTypeKind::General),
            2 => new FloatType(FloatTypeKind::UnspecifiedLiteral),
            3 => new FloatType(FloatTypeKind::Literal, $reader->readF64()),
            default => throw new ProtocolException("Unknown analyzer float type kind {$kind}."),
        };
    }

    private static function readString(PayloadReader $reader): StringType
    {
        $literalKind = $reader->readU8();
        $literalValue = $literalKind === 2 ? $reader->readBytes() : null;
        $bits = $reader->readU8();

        return new StringType(
            match ($literalKind) {
                0 => StringLiteralKind::General,
                1 => StringLiteralKind::Unspecified,
                2 => StringLiteralKind::Value,
                default => throw new ProtocolException("Unknown string literal kind {$literalKind}."),
            },
            $literalValue,
            ($bits & 1) !== 0,
            ($bits & (1 << 1)) !== 0,
            ($bits & (1 << 2)) !== 0,
            ($bits & (1 << 3)) !== 0,
            match ($casing = $reader->readU8()) {
                0 => StringCasing::Unspecified,
                1 => StringCasing::Lowercase,
                2 => StringCasing::Uppercase,
                default => throw new ProtocolException("Unknown string casing {$casing}."),
            },
        );
    }

    private static function readClassLikeString(PayloadReader $reader, bool $references): ClassLikeStringType
    {
        return match ($variant = $reader->readU8()) {
            1 => new ClassLikeStringType(ClassLikeStringVariant::Any, self::readClassLikeStringKind($reader)),
            2 => new ClassLikeStringType(
                ClassLikeStringVariant::Generic,
                self::readClassLikeStringKind($reader),
                parameterName: $reader->readBytes(),
                definingEntity: self::readGenericParent($reader),
                constraint: self::readAtomic($reader, $references),
            ),
            3 => new ClassLikeStringType(ClassLikeStringVariant::Literal, literal: $reader->readBytes()),
            4 => new ClassLikeStringType(
                ClassLikeStringVariant::OfType,
                self::readClassLikeStringKind($reader),
                constraint: self::readAtomic($reader, $references),
            ),
            default => throw new ProtocolException("Unknown class-like string variant {$variant}."),
        };
    }

    private static function readCallable(PayloadReader $reader, bool $references): CallableType
    {
        $kind = $reader->readU8();
        if ($kind === 2) {
            return new CallableType(null, self::readFunctionLikeIdentifier($reader));
        }
        if ($kind !== 1) {
            throw new ProtocolException("Unknown callable type kind {$kind}.");
        }

        $pure = $reader->readBoolean();
        $closure = $reader->readBoolean();
        $parameterCount = $reader->readCount(self::MAXIMUM_MEMBERS);
        $parameters = [];
        for ($index = 0; $index < $parameterCount; ++$index) {
            $name = $reader->readBoolean() ? $reader->readBytes() : null;
            $type = $reader->readBoolean() ? self::readNestedType($reader, $references) : null;
            $parameters[] = new CallableParameter(
                $name,
                $type,
                $reader->readBoolean() ? self::readNestedType($reader, $references) : null,
                $reader->readBoolean(),
                $reader->readBoolean(),
                $reader->readBoolean(),
            );
        }

        $returnType = $reader->readBoolean() ? self::readNestedType($reader, $references) : null;
        $source = $reader->readBoolean() ? self::readFunctionLikeIdentifier($reader) : null;
        $constraintCount = $reader->readCount(self::MAXIMUM_MEMBERS);
        $constraints = [];
        for ($index = 0; $index < $constraintCount; ++$index) {
            $nameCount = $reader->readCount(self::MAXIMUM_MEMBERS);
            $names = [];
            for ($nameIndex = 0; $nameIndex < $nameCount; ++$nameIndex) {
                $names[] = $reader->readBytes();
            }

            $constraints[] = new CallableConstraint(
                $names,
                self::readNestedType($reader, $references),
                self::readNestedType($reader, $references),
            );
        }

        return new CallableType(
            new CallableSignature($pure, $closure, $parameters, $returnType, $source, $constraints),
            null,
        );
    }

    private static function readMixed(PayloadReader $reader): MixedType
    {
        $bits = $reader->readU8();

        return new MixedType(($bits & 1) !== 0, ($bits & (1 << 1)) !== 0, ($bits & (1 << 2)) !== 0, match (
            $truthiness = $reader->readU8()
        ) {
            0 => MixedTruthiness::Undetermined,
            1 => MixedTruthiness::Truthy,
            2 => MixedTruthiness::Falsy,
            default => throw new ProtocolException("Unknown mixed truthiness {$truthiness}."),
        });
    }

    private static function readObject(PayloadReader $reader, bool $references): AtomicType
    {
        return match ($kind = $reader->readU8()) {
            1 => new AnyObjectType(),
            2 => new NamedObjectType(
                $reader->readBytes(),
                self::readOptionalTypes($reader, $references),
                self::readOptionalVariances($reader),
                $reader->readBoolean(),
                $reader->readBoolean(),
                self::readOptionalAtomics($reader, $references),
                $reader->readBoolean(),
            ),
            3 => new EnumType($reader->readBytes(), $reader->readOptionalString()),
            4 => self::readObjectShape($reader, $references),
            5 => new ObjectWithMethodType($reader->readBytes(), self::readOptionalAtomics($reader, $references)),
            6 => new ObjectWithPropertyType($reader->readBytes(), self::readOptionalAtomics($reader, $references)),
            default => throw new ProtocolException("Unknown object type kind {$kind}."),
        };
    }

    private static function readObjectShape(PayloadReader $reader, bool $references): ObjectShapeType
    {
        $sealed = $reader->readBoolean();
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $properties = [];
        for ($index = 0; $index < $count; ++$index) {
            $properties[] = new ObjectProperty(
                $reader->readBytes(),
                $reader->readBoolean(),
                self::readNestedType($reader, $references),
            );
        }

        return new ObjectShapeType($properties, $sealed);
    }

    private static function readArray(PayloadReader $reader, bool $references): AtomicType
    {
        return match ($kind = $reader->readU8()) {
            1 => self::readList($reader, $references),
            2 => self::readKeyedArray($reader, $references),
            default => throw new ProtocolException("Unknown array type kind {$kind}."),
        };
    }

    private static function readList(PayloadReader $reader, bool $references): ListType
    {
        $elementType = self::readNestedType($reader, $references);
        $elements = null;
        if ($reader->readBoolean()) {
            $count = $reader->readCount(self::MAXIMUM_MEMBERS);
            $elements = [];
            for ($index = 0; $index < $count; ++$index) {
                $elements[] = new ListElement(
                    $reader->readU64(),
                    $reader->readBoolean(),
                    self::readNestedType($reader, $references),
                );
            }
        }

        $knownCount = $reader->readBoolean() ? $reader->readU64() : null;

        return new ListType($elementType, $elements, $knownCount, $reader->readBoolean());
    }

    private static function readKeyedArray(PayloadReader $reader, bool $references): KeyedArrayType
    {
        $items = null;
        if ($reader->readBoolean()) {
            $count = $reader->readCount(self::MAXIMUM_MEMBERS);
            $items = [];
            for ($index = 0; $index < $count; ++$index) {
                $items[] = new ArrayItem(
                    self::readArrayKey($reader),
                    $reader->readBoolean(),
                    self::readNestedType($reader, $references),
                );
            }
        }

        $keyType = null;
        $valueType = null;
        if ($reader->readBoolean()) {
            $keyType = self::readNestedType($reader, $references);
            $valueType = self::readNestedType($reader, $references);
        }

        return new KeyedArrayType($items, $keyType, $valueType, $reader->readBoolean());
    }

    private static function readArrayKey(PayloadReader $reader): ArrayKey
    {
        return match ($kind = $reader->readU8()) {
            1 => new ArrayKey(ArrayKeyKind::Integer, $reader->readI64()),
            2 => new ArrayKey(ArrayKeyKind::String, $reader->readBytes()),
            3 => new ArrayKey(ArrayKeyKind::ClassLikeConstant, $reader->readBytes(), $reader->readBytes()),
            default => throw new ProtocolException("Unknown array key kind {$kind}."),
        };
    }

    private static function readReference(PayloadReader $reader, bool $references): ReferenceType
    {
        return match ($kind = $reader->readU8()) {
            1 => new ReferenceType(
                ReferenceTypeKind::Symbol,
                $reader->readBytes(),
                self::readOptionalTypes($reader, $references),
                self::readOptionalVariances($reader),
                self::readOptionalAtomics($reader, $references),
                null,
                null,
            ),
            2 => new ReferenceType(
                ReferenceTypeKind::Member,
                $reader->readBytes(),
                null,
                null,
                null,
                ...self::readReferenceSelector($reader),
            ),
            3 => new ReferenceType(
                ReferenceTypeKind::Global,
                null,
                null,
                null,
                null,
                ...self::readReferenceSelector($reader),
            ),
            default => throw new ProtocolException("Unknown reference type kind {$kind}."),
        };
    }

    /** @return array{?string, ReferenceSelectorKind} */
    private static function readReferenceSelector(PayloadReader $reader): array
    {
        return match ($kind = $reader->readU8()) {
            1 => [null, ReferenceSelectorKind::Wildcard],
            2 => [$reader->readBytes(), ReferenceSelectorKind::Identifier],
            3 => [$reader->readBytes(), ReferenceSelectorKind::StartsWith],
            4 => [$reader->readBytes(), ReferenceSelectorKind::EndsWith],
            default => throw new ProtocolException("Unknown reference selector kind {$kind}."),
        };
    }

    private static function readDerived(PayloadReader $reader, bool $references): DerivedType
    {
        return match ($kind = $reader->readU8()) {
            1 => new DerivedType(DerivedTypeKind::KeyOf, [self::readNestedType($reader, $references)]),
            2 => new DerivedType(DerivedTypeKind::ValueOf, [self::readNestedType($reader, $references)]),
            3 => new DerivedType(DerivedTypeKind::IntMask, self::readTypes($reader, $references)),
            4 => new DerivedType(DerivedTypeKind::IntMaskOf, [self::readNestedType($reader, $references)]),
            5 => self::readPropertiesOf($reader, $references),
            6 => new DerivedType(DerivedTypeKind::IndexAccess, [
                self::readNestedType($reader, $references),
                self::readNestedType($reader, $references),
            ]),
            7 => new DerivedType(DerivedTypeKind::New_, [self::readNestedType($reader, $references)]),
            8 => new DerivedType(DerivedTypeKind::TemplateType, [
                self::readNestedType($reader, $references),
                self::readNestedType($reader, $references),
                self::readNestedType($reader, $references),
            ]),
            9 => new DerivedType(
                DerivedTypeKind::Intersection,
                [self::readNestedType($reader, $references)],
                self::readAtomics($reader, $references),
            ),
            default => throw new ProtocolException("Unknown derived type kind {$kind}."),
        };
    }

    private static function readPropertiesOf(PayloadReader $reader, bool $references): DerivedType
    {
        $visibility = match ($kind = $reader->readU8()) {
            0 => null,
            1 => Visibility::Public,
            2 => Visibility::Protected,
            3 => Visibility::Private,
            default => throw new ProtocolException("Unknown properties-of visibility {$kind}."),
        };

        return new DerivedType(
            DerivedTypeKind::PropertiesOf,
            [self::readNestedType($reader, $references)],
            visibility: $visibility,
        );
    }

    private static function readClassLikeStringKind(PayloadReader $reader): ClassLikeStringKind
    {
        return match ($kind = $reader->readU8()) {
            1 => ClassLikeStringKind::Class_,
            2 => ClassLikeStringKind::Interface,
            3 => ClassLikeStringKind::Enum,
            4 => ClassLikeStringKind::Trait,
            default => throw new ProtocolException("Unknown class-like string kind {$kind}."),
        };
    }

    private static function readGenericParent(PayloadReader $reader): GenericParent
    {
        return match ($kind = $reader->readU8()) {
            1 => new GenericParent(GenericParentKind::ClassLike, $reader->readBytes()),
            2 => new GenericParent(GenericParentKind::FunctionLike, $reader->readBytes(), $reader->readBytes()),
            default => throw new ProtocolException("Unknown generic parent kind {$kind}."),
        };
    }

    public static function readFunctionLikeIdentifier(PayloadReader $reader): FunctionLikeIdentifier
    {
        return match ($kind = $reader->readU8()) {
            1 => new FunctionLikeIdentifier(FunctionLikeKind::Function_, $reader->readBytes()),
            2 => self::readMethodIdentifier($reader),
            3 => new FunctionLikeIdentifier(FunctionLikeKind::Closure, $reader->readBytes()),
            default => throw new ProtocolException("Unknown function-like identifier kind {$kind}."),
        };
    }

    private static function readMethodIdentifier(PayloadReader $reader): FunctionLikeIdentifier
    {
        $class = $reader->readBytes();

        return new FunctionLikeIdentifier(FunctionLikeKind::Method, $reader->readBytes(), $class);
    }

    /** @return null|list<Type> */
    private static function readOptionalTypes(PayloadReader $reader, bool $references): ?array
    {
        return $reader->readBoolean() ? self::readTypes($reader, $references) : null;
    }

    /** @return list<Type> */
    private static function readTypes(PayloadReader $reader, bool $references): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $types = [];
        for ($index = 0; $index < $count; ++$index) {
            $types[] = self::readNestedType($reader, $references);
        }

        return $types;
    }

    /** @return null|list<AtomicType> */
    private static function readOptionalAtomics(PayloadReader $reader, bool $references): ?array
    {
        return $reader->readBoolean() ? self::readAtomics($reader, $references) : null;
    }

    /** @return list<AtomicType> */
    private static function readAtomics(PayloadReader $reader, bool $references): array
    {
        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $types = [];
        for ($index = 0; $index < $count; ++$index) {
            $types[] = self::readAtomic($reader, $references);
        }

        return $types;
    }

    /** @return null|list<Variance> */
    private static function readOptionalVariances(PayloadReader $reader): ?array
    {
        if (!$reader->readBoolean()) {
            return null;
        }

        $count = $reader->readCount(self::MAXIMUM_MEMBERS);
        $variances = [];
        for ($index = 0; $index < $count; ++$index) {
            $variances[] = match ($variance = $reader->readU8()) {
                1 => Variance::Invariant,
                2 => Variance::Covariant,
                3 => Variance::Contravariant,
                4 => Variance::Bivariant,
                default => throw new ProtocolException("Unknown type variance {$variance}."),
            };
        }

        return $variances;
    }

    /** @param non-empty-list<AtomicType> $atomicTypes */
    private static function describeAtomics(array $atomicTypes): string
    {
        $description = '';
        foreach ($atomicTypes as $atomicType) {
            $description .= ($description === '' ? '' : '|') . (string) $atomicType;
        }

        return $description;
    }
}
