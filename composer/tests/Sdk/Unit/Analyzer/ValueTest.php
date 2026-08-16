<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\AttributedEntryPoint;
use Mago\Sdk\Analyzer\ClassTarget;
use Mago\Sdk\Analyzer\FunctionTarget;
use Mago\Sdk\Analyzer\MethodTarget;
use Mago\Sdk\Analyzer\PluginDefinition;
use Mago\Sdk\Analyzer\PropertyTarget;
use Mago\Sdk\Analyzer\PropertyType;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\ClassLikeStringType;
use Mago\Sdk\Analyzer\Type\ClassLikeStringVariant;
use Mago\Sdk\Analyzer\Type\FunctionLikeIdentifier;
use Mago\Sdk\Analyzer\Type\FunctionLikeKind;
use Mago\Sdk\Analyzer\Type\ScalarType;
use Mago\Sdk\Analyzer\Type\ScalarTypeKind;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Span;
use PHPUnit\Framework\TestCase;

final class ValueTest extends TestCase
{
    public function testInvalidSpanIsRejected(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new Span(2, 1);
    }

    public function testAnalyzerValuesRetainTheirData(): void
    {
        $plugin = new PluginDefinition('demo', 'Demo', 'Demo analyzer plugin.', ['example']);

        self::assertSame(['example'], $plugin->aliases);
        self::assertSame('demo', FunctionTarget::exact('demo')->value);
        self::assertSame('Model', ClassTarget::exact('Model')->class);
        self::assertSame('*', ClassTarget::any()->class);
        self::assertSame('*', MethodTarget::anyClass('create')->class);
        self::assertSame('*', PropertyTarget::allProperties('Model')->property);
        $entryPoint = new AttributedEntryPoint('FrameworkTestCase', 'FrameworkTest');
        self::assertSame('FrameworkTestCase', $entryPoint->class->class);
        self::assertSame('FrameworkTest', $entryPoint->attribute);

        $method = new FunctionLikeIdentifier(FunctionLikeKind::Method, 'run', 'Job');
        self::assertTrue($method->equals(new FunctionLikeIdentifier(FunctionLikeKind::Method, 'RUN', 'job')));
        self::assertFalse($method->equals(new FunctionLikeIdentifier(FunctionLikeKind::Method, 'stop', 'Job')));

        $property = new PropertyType(Type::string(), Type::int());
        self::assertSame('string', (string) $property->readType);
        self::assertSame('int', (string) $property->writeType);
    }

    public function testPropertyTypeRequiresAtLeastOneAccessType(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new PropertyType();
    }

    public function testPropertyTargetRejectsDollarPrefixedNames(): void
    {
        $this->expectException(InvalidArgumentException::class);

        PropertyTarget::exact('Model', '$name');
    }

    public function testClassTargetRejectsEmbeddedWildcards(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new ClassTarget('App\\*\\Model');
    }

    public function testAttributedEntryPointRejectsAnEmptyAttribute(): void
    {
        $this->expectException(InvalidArgumentException::class);

        new AttributedEntryPoint('FrameworkTestCase', '');
    }

    public function testTypeFactoriesBuildExpectedTypes(): void
    {
        self::assertSame('Box<string>', (string) Type::namedObject('Box', Type::string()));
        self::assertSame('string|null', (string) Type::union(Type::string(), Type::null()));
        self::assertSame('non-negative-int', (string) Type::nonNegativeInt());
        self::assertSame('non-empty-string', (string) Type::nonEmptyString());
        self::assertSame('int(-42)', (string) Type::literalInt(-42));
    }

    public function testLiteralValuesCanBeReadWithoutInspectingAtomicTypes(): void
    {
        $classString = Type::fromAtomics(
            new ScalarType(
                ScalarTypeKind::ClassLikeString,
                new ClassLikeStringType(ClassLikeStringVariant::Literal, literal: 'App\\Example'),
            ),
        );

        self::assertSame(-42, Type::literalInt(-42)->getLiteralInt());
        self::assertSame('value', Type::literalString('value')->getLiteralString());
        self::assertTrue(Type::true()->getLiteralBool());
        self::assertFalse(Type::false()->getLiteralBool());
        self::assertSame('App\\Example', $classString->getLiteralString());
        self::assertSame('App\\Example', $classString->getLiteralClassString());
        self::assertNull(Type::literalString('App\\Example')->getLiteralClassString());
        self::assertNull(Type::string()->getLiteralString());
        self::assertNull(Type::bool()->getLiteralBool());
    }
}
