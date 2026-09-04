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
use Mago\Sdk\Analyzer\InvocationAssertions;
use Mago\Sdk\Analyzer\Type\ArrayKey;
use Mago\Sdk\Analyzer\Type\ArrayKeyKind;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Protocol\PayloadWriter;

use function count;
use function is_int;
use function is_string;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:kan-defect
 */
final class AssertionCodec
{
    public static function write(PayloadWriter $writer, InvocationAssertions $assertions): void
    {
        self::writeMap($writer, $assertions->assertions);
        self::writeMap($writer, $assertions->ifTrueAssertions);
        self::writeMap($writer, $assertions->ifFalseAssertions);
    }

    /** @param array<string, list<Assertion>> $assertions */
    private static function writeMap(PayloadWriter $writer, array $assertions): void
    {
        $writer->writeCount($assertions);
        foreach ($assertions as $parameter => $facts) {
            $writer->writeBytes($parameter);
            $writer->writeCount($facts);
            foreach ($facts as $fact) {
                self::writeAssertion($writer, $fact);
            }
        }
    }

    private static function writeAssertion(PayloadWriter $writer, Assertion $assertion): void
    {
        $kind = match (true) {
            $assertion instanceof SimpleAssertion => match ($assertion->kind) {
                SimpleAssertionKind::Any => 1,
                SimpleAssertionKind::Falsy => 4,
                SimpleAssertionKind::Truthy => 5,
                SimpleAssertionKind::IsEqualIsset => 10,
                SimpleAssertionKind::IsIsset => 11,
                SimpleAssertionKind::IsNotIsset => 12,
                SimpleAssertionKind::HasStringArrayAccess => 13,
                SimpleAssertionKind::HasIntOrStringArrayAccess => 14,
                SimpleAssertionKind::ArrayKeyExists => 15,
                SimpleAssertionKind::ArrayKeyDoesNotExist => 16,
                SimpleAssertionKind::Empty => 23,
                SimpleAssertionKind::NonEmpty => 24,
                SimpleAssertionKind::EmptyCountable => 26,
                SimpleAssertionKind::Countable => 43,
            },
            $assertion instanceof TypeAssertion => match ($assertion->kind) {
                TypeAssertionKind::IsType => 2,
                TypeAssertionKind::IsNotType => 3,
                TypeAssertionKind::IsIdentical => 6,
                TypeAssertionKind::IsNotIdentical => 7,
                TypeAssertionKind::IsEqual => 8,
                TypeAssertionKind::IsNotEqual => 9,
                TypeAssertionKind::InArray => 17,
                TypeAssertionKind::NotInArray => 18,
            },
            $assertion instanceof ArrayKeyAssertion => match ($assertion->kind) {
                ArrayKeyAssertionKind::HasKey => 19,
                ArrayKeyAssertionKind::DoesNotHaveKey => 20,
                ArrayKeyAssertionKind::HasNonnullEntryForKey => 21,
                ArrayKeyAssertionKind::DoesNotHaveNonnullEntryForKey => 22,
            },
            $assertion instanceof CountabilityAssertion => match ($assertion->kind) {
                CountabilityAssertionKind::NonEmpty => 25,
                CountabilityAssertionKind::NotCountable => 44,
            },
            $assertion instanceof IntegerAssertion => match ($assertion->kind) {
                IntegerAssertionKind::HasExactCount => 27,
                IntegerAssertionKind::HasAtLeastCount => 28,
                IntegerAssertionKind::DoesNotHaveExactCount => 29,
                IntegerAssertionKind::DoesNotHaveAtLeastCount => 30,
                IntegerAssertionKind::IsLessThan => 31,
                IntegerAssertionKind::IsLessThanOrEqual => 32,
                IntegerAssertionKind::IsGreaterThan => 33,
                IntegerAssertionKind::IsGreaterThanOrEqual => 34,
                IntegerAssertionKind::IsLessThanFromBound => 35,
                IntegerAssertionKind::IsLessThanOrEqualFromBound => 36,
                IntegerAssertionKind::IsGreaterThanFromBound => 37,
                IntegerAssertionKind::IsGreaterThanOrEqualFromBound => 38,
                IntegerAssertionKind::StringLengthLessThan => 45,
                IntegerAssertionKind::StringLengthGreaterThanOrEqual => 46,
            },
            $assertion instanceof VariableAssertion => match ($assertion->kind) {
                VariableAssertionKind::IsLessThan => 39,
                VariableAssertionKind::IsLessThanOrEqual => 40,
                VariableAssertionKind::IsGreaterThan => 41,
                VariableAssertionKind::IsGreaterThanOrEqual => 42,
            },
            default => throw new ProtocolException('Cannot encode an unknown invocation assertion.'),
        };
        $writer->writeU8($kind);

        if ($assertion instanceof TypeAssertion) {
            if (
                $assertion->kind !== TypeAssertionKind::InArray
                && $assertion->kind !== TypeAssertionKind::NotInArray
                && count($assertion->type->atomicTypes) !== 1
            ) {
                throw new ProtocolException("Assertion `{$assertion->kind->name}` requires exactly one atomic type.");
            }

            $writer->writeRaw($assertion->type->encode());
            return;
        }

        if ($assertion instanceof ArrayKeyAssertion) {
            self::writeArrayKey($writer, $assertion->key);
            return;
        }

        if ($assertion instanceof CountabilityAssertion) {
            $writer->writeBoolean($assertion->negatable);
            return;
        }

        if ($assertion instanceof IntegerAssertion) {
            if ($assertion->kind->isCount()) {
                $writer->writeU64($assertion->value);
                return;
            }

            $writer->writeI64($assertion->value);
            return;
        }

        if ($assertion instanceof VariableAssertion) {
            $writer->writeBytes($assertion->variable);
        }
    }

    private static function writeArrayKey(PayloadWriter $writer, ArrayKey $key): void
    {
        $writer->writeU8(match ($key->kind) {
            ArrayKeyKind::Integer => 1,
            ArrayKeyKind::String => 2,
            ArrayKeyKind::ClassLikeConstant => 3,
        });

        if ($key->kind === ArrayKeyKind::Integer) {
            if (!is_int($key->value)) {
                throw new ProtocolException('An integer assertion array key must contain an integer.');
            }

            $writer->writeI64($key->value);
            return;
        }

        if ($key->kind === ArrayKeyKind::String) {
            if (!is_string($key->value)) {
                throw new ProtocolException('A string assertion array key must contain a string.');
            }

            $writer->writeBytes($key->value);
            return;
        }

        self::writeClassLikeConstant($writer, $key);
    }

    private static function writeClassLikeConstant(PayloadWriter $writer, ArrayKey $key): void
    {
        if (!is_string($key->value) || $key->constant === null) {
            throw new ProtocolException('A class-like constant assertion array key is incomplete.');
        }

        $class = $key->value;
        $constant = $key->constant;
        $writer->writeBytes($class);
        $writer->writeBytes($constant);
    }
}
