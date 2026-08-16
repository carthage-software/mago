<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\Metadata\AttributeArgumentMetadata;
use Mago\Sdk\Analyzer\Metadata\AttributeMetadata;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\SourceLocation;
use Mago\Sdk\Span;
use PHPUnit\Framework\TestCase;

final class AttributeMetadataTest extends TestCase
{
    public function testArgumentsCanBeReadByPositionOrName(): void
    {
        $location = new SourceLocation('example.php', new Span(10, 20));
        $positional = new AttributeArgumentMetadata(null, $location, null, $location, Type::literalString('provider'));
        $named = new AttributeArgumentMetadata('enabled', $location, $location, $location, Type::false());
        $attribute = new AttributeMetadata('Example', $location, [$positional, $named]);

        self::assertSame($positional, $attribute->getArgument(0, 'methodName'));
        self::assertSame($named, $attribute->getArgument(1, 'enabled'));
        self::assertNull($attribute->getArgument(1));
        self::assertSame($positional, $attribute->getArgument(0, 'missing'));
        self::assertNull($attribute->getArgument(1, 'missing'));
    }
}
