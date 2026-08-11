<?php

declare(strict_types=1);

namespace Mago\Sdk\Analyzer;

use Mago\Sdk\Analyzer\Type\AnyObjectType;
use Mago\Sdk\Analyzer\Type\AtomicType;
use Mago\Sdk\Analyzer\Type\IntegerType;
use Mago\Sdk\Analyzer\Type\IntegerTypeKind;
use Mago\Sdk\Analyzer\Type\KeyedArrayType;
use Mago\Sdk\Analyzer\Type\ListType;
use Mago\Sdk\Analyzer\Type\MixedTruthiness;
use Mago\Sdk\Analyzer\Type\MixedType;
use Mago\Sdk\Analyzer\Type\NamedObjectType;
use Mago\Sdk\Analyzer\Type\ScalarType;
use Mago\Sdk\Analyzer\Type\ScalarTypeKind;
use Mago\Sdk\Analyzer\Type\SimpleAtomicType;
use Mago\Sdk\Analyzer\Type\SimpleAtomicTypeKind;
use Mago\Sdk\Analyzer\Type\StringCasing;
use Mago\Sdk\Analyzer\Type\StringLiteralKind;
use Mago\Sdk\Analyzer\Type\StringType;
use Mago\Sdk\Analyzer\Type\TypeFlags;
use Mago\Sdk\Exception\InvalidArgumentException;

use Mago\Sdk\Internal\Analyzer\TypeCodec;
use function count;
use function implode;
use function pack;
use function strlen;

/**
 * An immutable semantic type returned by an analyzer provider.
 *
 * @api
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:too-many-methods
 */
final class Type
{
    private const REFERENCE = 0;
    private const MIXED = 1;
    private const NEVER = 2;
    private const NULL = 3;
    private const VOID = 4;
    private const BOOL = 5;
    private const TRUE = 6;
    private const FALSE = 7;
    private const INT = 8;
    private const FLOAT = 9;
    private const STRING = 10;
    private const LITERAL_STRING = 11;
    private const OBJECT = 12;
    private const NAMED_OBJECT = 13;
    private const ARRAY = 14;
    private const LIST = 15;
    private const UNION = 16;
    private const NON_NEGATIVE_INT = 17;
    private const NON_EMPTY_STRING = 18;
    private const LITERAL_INT = 19;
    private const COMPLETE = 20;

    /**
     * @param non-empty-list<AtomicType> $atomicTypes
     */
    private function __construct(
        private readonly string $payload,
        private readonly string $description,
        public readonly array $atomicTypes,
        public readonly TypeFlags $flags,
    ) {}

    public static function mixed(): self
    {
        return self::simple(self::MIXED, 'mixed', new MixedType(false, false, false, MixedTruthiness::Undetermined));
    }

    public static function never(): self
    {
        return self::simple(self::NEVER, 'never', new SimpleAtomicType(SimpleAtomicTypeKind::Never));
    }

    public static function null(): self
    {
        return self::simple(self::NULL, 'null', new SimpleAtomicType(SimpleAtomicTypeKind::Null));
    }

    public static function void(): self
    {
        return self::simple(self::VOID, 'void', new SimpleAtomicType(SimpleAtomicTypeKind::Void));
    }

    public static function bool(): self
    {
        return self::simple(self::BOOL, 'bool', new ScalarType(ScalarTypeKind::Boolean));
    }

    public static function true(): self
    {
        return self::simple(self::TRUE, 'true', new ScalarType(ScalarTypeKind::Boolean, true));
    }

    public static function false(): self
    {
        return self::simple(self::FALSE, 'false', new ScalarType(ScalarTypeKind::Boolean, false));
    }

    public static function int(): self
    {
        return self::simple(
            self::INT,
            'int',
            new ScalarType(ScalarTypeKind::Integer, new IntegerType(IntegerTypeKind::General)),
        );
    }

    public static function literalInt(int $value): self
    {
        return new self(
            pack('CJ', self::LITERAL_INT, $value),
            "int({$value})",
            [new ScalarType(ScalarTypeKind::Integer, new IntegerType(IntegerTypeKind::Literal, $value, $value))],
            new TypeFlags(),
        );
    }

    public static function nonNegativeInt(): self
    {
        return self::simple(
            self::NON_NEGATIVE_INT,
            'non-negative-int',
            new ScalarType(ScalarTypeKind::Integer, new IntegerType(IntegerTypeKind::From, 0)),
        );
    }

    public static function float(): self
    {
        return self::simple(self::FLOAT, 'float', new ScalarType(ScalarTypeKind::Float));
    }

    public static function string(): self
    {
        return self::simple(
            self::STRING,
            'string',
            new ScalarType(
                ScalarTypeKind::String,
                new StringType(StringLiteralKind::General, null, false, false, false, false, StringCasing::Unspecified),
            ),
        );
    }

    public static function nonEmptyString(): self
    {
        return self::simple(
            self::NON_EMPTY_STRING,
            'non-empty-string',
            new ScalarType(
                ScalarTypeKind::String,
                new StringType(StringLiteralKind::General, null, false, false, true, false, StringCasing::Unspecified),
            ),
        );
    }

    public static function object(): self
    {
        return self::simple(self::OBJECT, 'object', new AnyObjectType());
    }

    public static function literalString(string $value): self
    {
        return new self(
            pack('CN', self::LITERAL_STRING, strlen($value)) . $value,
            "'{$value}'",
            [new ScalarType(
                ScalarTypeKind::String,
                new StringType(
                    StringLiteralKind::Value,
                    $value,
                    false,
                    $value !== '' && $value !== '0',
                    $value !== '',
                    false,
                    StringCasing::Unspecified,
                ),
            )],
            new TypeFlags(),
        );
    }

    public static function namedObject(string $class, self ...$parameters): self
    {
        if ($class === '') {
            throw new InvalidArgumentException('A named object type requires a class name.');
        }

        $payload = pack('CN', self::NAMED_OBJECT, strlen($class)) . $class . pack('N', count($parameters));
        $descriptions = [];
        foreach ($parameters as $parameter) {
            $payload .= $parameter->payload;
            $descriptions[] = $parameter->description;
        }

        $description = $class;
        if ($descriptions !== []) {
            $description .= '<' . implode(', ', $descriptions) . '>';
        }

        return new self(
            $payload,
            $description,
            [new NamedObjectType($class, [...$parameters], null, false, false, null, false)],
            new TypeFlags(),
        );
    }

    public static function array(self $key, self $value): self
    {
        return new self(
            pack('C', self::ARRAY) . $key->payload . $value->payload,
            "array<{$key->description}, {$value->description}>",
            [new KeyedArrayType(null, $key, $value, false)],
            new TypeFlags(),
        );
    }

    public static function list(self $element): self
    {
        return new self(
            pack('C', self::LIST) . $element->payload,
            "list<{$element->description}>",
            [new ListType($element, null, null, false)],
            new TypeFlags(),
        );
    }

    public static function union(self $first, self $second, self ...$additional): self
    {
        $members = [$first, $second, ...$additional];
        $payload = pack('CN', self::UNION, count($members));
        $descriptions = [];
        $atomicTypes = [];
        foreach ($members as $member) {
            $payload .= $member->payload;
            $descriptions[] = $member->description;
            foreach ($member->atomicTypes as $atomicType) {
                $atomicTypes[] = $atomicType;
            }
        }

        return new self($payload, implode('|', $descriptions), $atomicTypes, new TypeFlags());
    }

    public static function fromAtomic(AtomicType $atomicType, ?TypeFlags $flags = null): self
    {
        return new self('', (string) $atomicType, [$atomicType], $flags ?? new TypeFlags());
    }

    public static function fromAtomics(AtomicType $first, AtomicType ...$additional): self
    {
        $atomicTypes = [$first, ...$additional];

        return new self('', self::describeAtomics($atomicTypes), $atomicTypes, new TypeFlags());
    }

    public function withFlags(TypeFlags $flags): self
    {
        if ($flags === $this->flags) {
            return $this;
        }

        return new self('', $this->description, $this->atomicTypes, $flags);
    }

    /**
     * @param int<0, 4294967295> $handle
     * @param non-empty-list<AtomicType> $atomicTypes
     * @internal
     */
    public static function reference(
        int $handle,
        string $description,
        array $atomicTypes,
        TypeFlags $flags,
    ): self {
        return new self(pack('CN', self::REFERENCE, $handle), $description, $atomicTypes, $flags);
    }

    public function getLiteralInt(): ?int
    {
        if (count($this->atomicTypes) !== 1) {
            return null;
        }

        $atomic = $this->atomicTypes[0];
        if (!$atomic instanceof ScalarType || !$atomic->refinement instanceof IntegerType) {
            return null;
        }

        return $atomic->refinement->getLiteralValue();
    }

    public function getLiteralString(): ?string
    {
        if (count($this->atomicTypes) !== 1) {
            return null;
        }

        $atomic = $this->atomicTypes[0];
        if (!$atomic instanceof ScalarType || !$atomic->refinement instanceof StringType) {
            return null;
        }

        return $atomic->refinement->literalKind === StringLiteralKind::Value ? $atomic->refinement->literalValue : null;
    }

    /** @internal */
    public function encode(): string
    {
        if ($this->payload !== '') {
            return $this->payload;
        }

        return pack('C', self::COMPLETE) . TypeCodec::encode($this);
    }

    public function __toString(): string
    {
        return $this->description;
    }

    private static function simple(int $kind, string $description, AtomicType $atomicType): self
    {
        return new self(pack('C', $kind), $description, [$atomicType], new TypeFlags());
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
