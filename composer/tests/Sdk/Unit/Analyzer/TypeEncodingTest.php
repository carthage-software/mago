<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Analyzer\Type\ListType;
use Mago\Sdk\Analyzer\Type\ReferenceSelectorKind;
use Mago\Sdk\Analyzer\Type\ReferenceType;
use Mago\Sdk\Analyzer\Type\ReferenceTypeKind;
use Mago\Sdk\Analyzer\Type\TypeFlags;
use Mago\Sdk\Internal\Analyzer\TypeCodec;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use PHPUnit\Framework\TestCase;

final class TypeEncodingTest extends TestCase
{
    public function testGlobalConstantFallbackRoundTrip(): void
    {
        $reference = new ReferenceType(
            ReferenceTypeKind::Global,
            null,
            null,
            null,
            null,
            'app\MODE_OBJECT',
            ReferenceSelectorKind::Fallback,
            'MODE_OBJECT',
        );
        $type = Type::fromAtomic($reference);
        $writer = new PayloadWriter();
        $writer->writeU32(0);
        TypeCodec::writeComplete($writer, $type);
        $reader = new PayloadReader($writer->finish());
        $decoded = TypeCodec::readComplete($reader);
        $reader->finish();

        self::assertEquals($reference, $decoded->atomicTypes[0]);
        self::assertSame($type->encode(), $decoded->encode());
    }

    public function testStructuredTypeUsesCompleteEncoding(): void
    {
        $type = Type::fromAtomic(new ListType(Type::string(), null, null, true))->withFlags(
            new TypeFlags(possiblyUndefined: true),
        );
        $reader = new PayloadReader($type->encode());

        self::assertSame(20, $reader->readU8());
        self::assertSame(1 << 4, $reader->readU16());
        self::assertSame(1, $reader->readU32());
        self::assertSame(5, $reader->readU8());
        self::assertSame(1, $reader->readU8());
        self::assertSame(0, $reader->readU16());
        self::assertSame(1, $reader->readU32());
        self::assertSame(1, $reader->readU8());
        self::assertSame(7, $reader->readU8());
        self::assertSame(0, $reader->readU8());
        self::assertSame(0, $reader->readU8());
        self::assertSame(0, $reader->readU8());
        self::assertFalse($reader->readBoolean());
        self::assertFalse($reader->readBoolean());
        self::assertTrue($reader->readBoolean());
        $reader->finish();
    }
}
