<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Protocol;

use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use PHPUnit\Framework\TestCase;

use function pack;

final class PayloadTest extends TestCase
{
    public function testUnsignedValuesRoundTrip(): void
    {
        $writer = new PayloadWriter();
        $writer->writeU8(7);
        $writer->writeU16(0x0203);
        $writer->writeU32(0x0405_0607);
        $writer->writeU64(42);
        $writer->writeBoolean(true);
        $writer->writeBytes("bytes\0");
        $writer->writeString('string');
        $writer->writeOptionalString('optional');
        $writer->writeOptionalString(null);

        $reader = new PayloadReader($writer->finish());
        self::assertSame(7, $reader->readU8());
        self::assertSame(0x0203, $reader->readU16());
        self::assertSame(0x0405_0607, $reader->readU32());
        self::assertSame(42, $reader->readU64());
        self::assertTrue($reader->readBoolean());
        self::assertSame("bytes\0", $reader->readBytes());
        self::assertSame('string', $reader->readString());
        self::assertSame('optional', $reader->readOptionalString());
        self::assertNull($reader->readOptionalString());
        $reader->finish();
    }

    public function testSignedIntegerRoundTrips(): void
    {
        $reader = new PayloadReader(pack('J', -42));

        self::assertSame(-42, $reader->readI64());
        $reader->finish();
    }

    public function testFloatRoundTrips(): void
    {
        $reader = new PayloadReader(pack('E', 3.5));

        self::assertSame(3.5, $reader->readF64());
        $reader->finish();
    }
}
