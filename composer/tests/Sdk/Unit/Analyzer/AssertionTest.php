<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\Assertion\ArrayKeyAssertion;
use Mago\Sdk\Analyzer\Assertion\ArrayKeyAssertionKind;
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
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\ArrayKey;
use Mago\Sdk\Analyzer\Type\ArrayKeyKind;
use Mago\Sdk\Exception\InvalidArgumentException;
use PHPUnit\Framework\TestCase;

final class AssertionTest extends TestCase
{
    public function testAssertionVariantsRetainTypedPayloads(): void
    {
        $simple = new SimpleAssertion(SimpleAssertionKind::Truthy);
        $type = new TypeAssertion(TypeAssertionKind::IsType, Type::string());
        $key = new ArrayKeyAssertion(
            ArrayKeyAssertionKind::HasNonnullEntryForKey,
            new ArrayKey(ArrayKeyKind::String, 'name'),
        );
        $integer = new IntegerAssertion(IntegerAssertionKind::IsGreaterThan, -1);
        $variable = new VariableAssertion(VariableAssertionKind::IsLessThan, '$limit');
        $countability = new CountabilityAssertion(CountabilityAssertionKind::NonEmpty, false);

        self::assertSame(SimpleAssertionKind::Truthy, $simple->kind);
        self::assertSame('string', (string) $type->type);
        self::assertSame('name', $key->key->value);
        self::assertSame(-1, $integer->value);
        self::assertSame('$limit', $variable->variable);
        self::assertFalse($countability->negatable);
    }

    public function testCountAssertionsRejectNegativeValues(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new IntegerAssertion(IntegerAssertionKind::HasExactCount, -1);
    }

    public function testVariableAssertionsRejectEmptyNames(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new VariableAssertion(VariableAssertionKind::IsGreaterThan, '');
    }

    public function testInvocationAssertionsRetainImmediateAndConditionalFacts(): void
    {
        $string = new TypeAssertion(TypeAssertionKind::IsType, Type::string());
        $assertions = new InvocationAssertions(
            assertions: ['$value' => [$string]],
            ifTrueAssertions: ['$truthy' => [new SimpleAssertion(SimpleAssertionKind::Truthy)]],
            ifFalseAssertions: ['$falsy' => [new SimpleAssertion(SimpleAssertionKind::Falsy)]],
        );

        self::assertSame(['$value' => [$string]], $assertions->assertions);
        self::assertFalse($assertions->isEmpty());
    }

    public function testInvocationAssertionsRejectEmptyFactLists(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new InvocationAssertions(assertions: ['$value' => []]);
    }
}
