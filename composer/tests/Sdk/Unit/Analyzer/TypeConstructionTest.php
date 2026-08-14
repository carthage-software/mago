<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\ArrayKey;
use Mago\Sdk\Analyzer\Type\ArrayKeyKind;
use Mago\Sdk\Analyzer\Type\CallableType;
use Mago\Sdk\Analyzer\Type\FloatType;
use Mago\Sdk\Analyzer\Type\FloatTypeKind;
use Mago\Sdk\Analyzer\Type\FunctionLikeIdentifier;
use Mago\Sdk\Analyzer\Type\FunctionLikeKind;
use Mago\Sdk\Analyzer\Type\GenericParent;
use Mago\Sdk\Analyzer\Type\GenericParentKind;
use Mago\Sdk\Analyzer\Type\IntegerType;
use Mago\Sdk\Analyzer\Type\IntegerTypeKind;
use Mago\Sdk\Analyzer\Type\KeyedArrayType;
use Mago\Sdk\Analyzer\Type\ListElement;
use Mago\Sdk\Analyzer\Type\ListType;
use Mago\Sdk\Analyzer\Type\NamedObjectType;
use Mago\Sdk\Analyzer\Type\StringCasing;
use Mago\Sdk\Analyzer\Type\StringLiteralKind;
use Mago\Sdk\Analyzer\Type\StringType;
use Mago\Sdk\Exception\InvalidArgumentException;
use PHPUnit\Framework\TestCase;

final class TypeConstructionTest extends TestCase
{
    public function testInvalidStructuredTypesFailBeforeEncoding(): void
    {
        $invalid = [
            static fn(): object => new IntegerType(IntegerTypeKind::Literal),
            static fn(): object => new IntegerType(IntegerTypeKind::Range, 2, 1),
            static fn(): object => new IntegerType(IntegerTypeKind::General, 0),
            static fn(): object => new FloatType(FloatTypeKind::Literal),
            static fn(): object => new FloatType(FloatTypeKind::General, 1.0),
            static fn(): object => new StringType(
                StringLiteralKind::Value,
                null,
                false,
                false,
                false,
                false,
                StringCasing::Unspecified,
            ),
            static fn(): object => new ArrayKey(ArrayKeyKind::Integer, '1'),
            static fn(): object => new ArrayKey(ArrayKeyKind::ClassLikeConstant, 'Example', ''),
            static fn(): object => new CallableType(null, null),
            static fn(): object => new KeyedArrayType(null, Type::int(), null, false),
            static fn(): object => new ListElement(-1, false, Type::mixed()),
            static fn(): object => new ListType(Type::mixed(), null, -1, false),
            static fn(): object => new NamedObjectType('', null, null, false, false, null, false),
            static fn(): object => new FunctionLikeIdentifier(FunctionLikeKind::Method, 'run'),
            static fn(): object => new FunctionLikeIdentifier(FunctionLikeKind::Function_, 'run', 'Example'),
            static fn(): object => new GenericParent(GenericParentKind::ClassLike, ''),
            static fn(): object => new GenericParent(GenericParentKind::ClassLike, 'Example', 'run'),
            static fn(): object => new GenericParent(GenericParentKind::FunctionLike, 'run'),
            static fn(): object => new GenericParent(GenericParentKind::FunctionLike, 'Example', ''),
        ];

        foreach ($invalid as $construct) {
            try {
                $construct();
                self::fail('Invalid structured type construction should throw.');
            } catch (InvalidArgumentException $exception) {
                self::assertNotSame('', $exception->getMessage());
            }
        }
    }

    public function testGlobalFunctionGenericParentAllowsAnEmptyClassComponent(): void
    {
        $parent = new GenericParent(GenericParentKind::FunctionLike, '', 'run');

        self::assertSame('', $parent->name);
        self::assertSame('run', $parent->member);
    }
}
