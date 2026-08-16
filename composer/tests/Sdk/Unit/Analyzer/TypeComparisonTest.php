<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\TypeComparison;
use Mago\Sdk\Analyzer\TypeComparisonKind;
use PHPUnit\Framework\TestCase;

final class TypeComparisonTest extends TestCase
{
    public function testFactoriesPreserveTheRequestedRelationship(): void
    {
        $integer = Type::int();
        $string = Type::string();
        $equal = TypeComparison::equal($integer, $string);
        $contained = TypeComparison::containedBy($integer, $string);
        $identical = TypeComparison::canBeIdentical($integer, $string);

        self::assertSame(TypeComparisonKind::Equal, $equal->kind);
        self::assertSame(TypeComparisonKind::ContainedBy, $contained->kind);
        self::assertSame(TypeComparisonKind::CanBeIdentical, $identical->kind);
        self::assertSame($integer, $contained->left);
        self::assertSame($string, $contained->right);
        self::assertNotSame($equal->cacheKey(), $contained->cacheKey());
        self::assertNotSame($contained->cacheKey(), $identical->cacheKey());
    }

    public function testEncodingsAreMemoized(): void
    {
        $comparison = TypeComparison::equal(Type::array(Type::int(), Type::string()), Type::mixed());

        self::assertSame($comparison->encodeLeft(), $comparison->encodeLeft());
        self::assertSame($comparison->encodeRight(), $comparison->encodeRight());
        self::assertSame($comparison->cacheKey(), $comparison->cacheKey());
    }
}
