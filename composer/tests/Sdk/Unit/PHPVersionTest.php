<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\PHPVersion;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;

final class PHPVersionTest extends TestCase
{
    public function testPackedIdentifierIsDecoded(): void
    {
        $version = new PHPVersion(0x08_05_11);

        self::assertSame(0x08_05_11, $version->id);
        self::assertSame(8, $version->major());
        self::assertSame(5, $version->minor());
        self::assertSame(17, $version->patch());
        self::assertSame('8.5.17', (string) $version);
    }

    public function testPartsArePackedForTheWire(): void
    {
        $version = PHPVersion::fromParts(8, 4, 2);

        self::assertSame(0x08_04_02, $version->id);
        self::assertSame('8.4.2', (string) $version);
    }

    public function testSemanticOrderingMatchesPackedOrdering(): void
    {
        self::assertTrue(PHPVersion::fromParts(8, 5)->isAtLeast(PHPVersion::fromParts(8, 4, 255)));
        self::assertFalse(PHPVersion::fromParts(8, 4, 1)->isAtLeast(PHPVersion::fromParts(8, 4, 2)));
    }

    #[DataProvider('provideInvalidIdentifiers')]
    public function testInvalidIdentifiersAreRejected(int $identifier): void
    {
        $this->expectException(InvalidArgumentException::class);

        new PHPVersion($identifier);
    }

    /** @return iterable<array{int}> */
    public static function provideInvalidIdentifiers(): iterable
    {
        yield 'negative' => [-1];
        yield 'larger than u32' => [0x1_0000_0000];
    }

    #[DataProvider('provideInvalidParts')]
    public function testInvalidPartsAreRejected(int $major, int $minor, int $patch): void
    {
        $this->expectException(InvalidArgumentException::class);

        PHPVersion::fromParts($major, $minor, $patch);
    }

    /** @return iterable<array{int, int, int}> */
    public static function provideInvalidParts(): iterable
    {
        yield 'negative major' => [-1, 0, 0];
        yield 'major overflow' => [65_536, 0, 0];
        yield 'negative minor' => [8, -1, 0];
        yield 'minor overflow' => [8, 256, 0];
        yield 'negative patch' => [8, 5, -1];
        yield 'patch overflow' => [8, 5, 256];
    }
}
