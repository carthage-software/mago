<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\Metadata\MetadataFlags;
use Mago\Sdk\Analyzer\Metadata\PropertyFields;
use Mago\Sdk\Analyzer\Type;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Analyzer\MetadataCodec;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\Analyzer\TypeCodec;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;

use function pack;
use function strlen;

/**
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:too-many-methods
 */
final class PropertyProjectionTest extends TestCase
{
    public function testRequestPreservesPropertyNamesAndOrder(): void
    {
        $properties = [new MemberIdentifier('App\\A', '$Name'), new MemberIdentifier('App\\B', '$name')];
        $fields = PropertyFields::NAMES | PropertyFields::FLAGS;
        [$kind, $reader] = Protocol::readRequest(Protocol::writeCodebasePropertyProjectionsRequest(
            7,
            $properties,
            $fields,
        ));

        self::assertSame(Protocol::CODEBASE_QUERY_REQUEST, $kind);
        self::assertSame(7, $reader->readU64());
        self::assertSame(20, $reader->readU8());
        self::assertSame($fields, $reader->readU32());
        self::assertSame(2, $reader->readCount(2));
        self::assertSame('App\\A', $reader->readBytes());
        self::assertSame('$Name', $reader->readBytes());
        self::assertSame('App\\B', $reader->readBytes());
        self::assertSame('$name', $reader->readBytes());
        $reader->finish();
    }

    public function testAllFieldsMatchExistingPropertyMetadata(): void
    {
        $payload = self::metadata(PropertyFields::ALL);
        $fullReader = new PayloadReader($payload);
        $full = MetadataCodec::readProperty($fullReader);
        $fullReader->finish();
        $projectionReader = new PayloadReader($payload);
        $projection = MetadataCodec::readPropertyProjection($projectionReader, PropertyFields::ALL);
        $projectionReader->finish();

        self::assertTrue($projection->has(PropertyFields::ALL));
        self::assertEquals($full->name, $projection->name, 'name');
        self::assertEquals($full->location, $projection->location, 'location');
        self::assertEquals($full->nameLocation, $projection->nameLocation, 'nameLocation');
        self::assertEquals($full->readVisibility, $projection->readVisibility, 'readVisibility');
        self::assertEquals($full->writeVisibility, $projection->writeVisibility, 'writeVisibility');
        self::assertEquals($full->declaredType, $projection->declaredType, 'declaredType');
        self::assertEquals($full->type, $projection->type, 'type');
        self::assertEquals($full->writeType, $projection->writeType, 'writeType');
        self::assertEquals($full->defaultType, $projection->defaultType, 'defaultType');
        self::assertEquals($full->attributes, $projection->attributes, 'attributes');
        self::assertEquals($full->flags, $projection->flags, 'flags');
        self::assertEquals($full->hooks, $projection->hooks, 'hooks');
        self::assertEquals($full->availableVersions, $projection->availableVersions, 'availableVersions');
        self::assertNotNull($projection->type);
        self::assertCount(1, $projection->attributes ?? []);
        self::assertArrayHasKey('get', $projection->hooks ?? []);
        self::assertCount(1, $projection->availableVersions ?? []);
    }

    /** @return iterable<string, array{int, string}> */
    public static function fieldGroups(): iterable
    {
        yield 'names' => [PropertyFields::NAMES, 'name'];
        yield 'locations' => [PropertyFields::LOCATIONS, 'location'];
        yield 'visibility' => [PropertyFields::VISIBILITY, 'readVisibility'];
        yield 'types' => [PropertyFields::TYPES, 'type'];
        yield 'attributes' => [PropertyFields::ATTRIBUTES, 'attributes'];
        yield 'flags' => [PropertyFields::FLAGS, 'flags'];
        yield 'hooks' => [PropertyFields::HOOKS, 'hooks'];
        yield 'versions' => [PropertyFields::AVAILABLE_VERSIONS, 'availableVersions'];
    }

    #[DataProvider('fieldGroups')]
    public function testEachFieldGroupCanBeDecodedIndependently(int $fields, string $selected): void
    {
        $reader = new PayloadReader(self::metadata($fields));
        $projection = MetadataCodec::readPropertyProjection($reader, $fields);
        $reader->finish();

        self::assertTrue($projection->has($fields));
        self::assertFalse($projection->has(PropertyFields::ALL));
        self::assertNotNull(match ($selected) {
            'name' => $projection->name,
            'location' => $projection->location,
            'readVisibility' => $projection->readVisibility,
            'type' => $projection->type,
            'attributes' => $projection->attributes,
            'flags' => $projection->flags,
            'hooks' => $projection->hooks,
            'availableVersions' => $projection->availableVersions,
            default => null,
        });
        if ($fields !== PropertyFields::NAMES) {
            self::assertNull($projection->name);
        }
        if ($fields !== PropertyFields::TYPES) {
            self::assertNull($projection->type);
        }
        if ($fields === PropertyFields::TYPES) {
            self::assertNull($projection->defaultType);
        }
    }

    public function testZeroFieldsRequireNoMetadataBytes(): void
    {
        $reader = new PayloadReader('');
        $projection = MetadataCodec::readPropertyProjection($reader, 0);
        $reader->finish();

        self::assertSame(0, $projection->fields);
        self::assertFalse($projection->has(PropertyFields::NAMES));
        self::assertNull($projection->name, 'name');
        self::assertNull($projection->location, 'location');
        self::assertNull($projection->nameLocation, 'nameLocation');
        self::assertNull($projection->readVisibility, 'readVisibility');
        self::assertNull($projection->writeVisibility, 'writeVisibility');
        self::assertNull($projection->declaredType, 'declaredType');
        self::assertNull($projection->type, 'type');
        self::assertNull($projection->writeType, 'writeType');
        self::assertNull($projection->defaultType, 'defaultType');
        self::assertNull($projection->attributes, 'attributes');
        self::assertNull($projection->flags, 'flags');
        self::assertNull($projection->hooks, 'hooks');
        self::assertNull($projection->availableVersions, 'availableVersions');
    }

    public function testSelectivePayloadOmitsUnrequestedGroups(): void
    {
        $fields = PropertyFields::NAMES | PropertyFields::LOCATIONS | PropertyFields::FLAGS;
        $selected = self::metadata($fields);
        $full = self::metadata(PropertyFields::ALL);
        $reader = new PayloadReader($selected);
        $projection = MetadataCodec::readPropertyProjection($reader, $fields);
        $reader->finish();

        self::assertTrue($projection->has($fields));
        self::assertSame('$value', $projection->name);
        self::assertNull($projection->attributes);
        self::assertNull($projection->hooks);
        self::assertSame(205, strlen($full));
        self::assertSame(48, strlen($selected));
        self::assertLessThan(strlen($full), strlen($selected));
    }

    /** @return iterable<string, array{string}> */
    public static function invalidResponses(): iterable
    {
        yield 'generation' => [self::response(8, 20, PropertyFields::NAMES)];
        yield 'operation' => [self::response(7, 5, PropertyFields::NAMES)];
        yield 'fields' => [self::response(7, 20, PropertyFields::ALL)];
        yield 'kind' => [Protocol::writeCodebasePropertyProjectionsRequest(7, [], PropertyFields::NAMES)];
    }

    #[DataProvider('invalidResponses')]
    public function testResponseHeaderMustMatchRequest(string $payload): void
    {
        $this->expectException(ProtocolException::class);

        Protocol::readCodebasePropertyProjectionsResponse($payload, 7, PropertyFields::NAMES);
    }

    private static function response(int $generation, int $operation, int $fields): string
    {
        $writer = new PayloadWriter(pack('N3', 0x4D41_4E41, 0x0002_0002, 0x8004 << 16));
        $writer->writeU64($generation);
        $writer->writeU8($operation);
        $writer->writeU32($fields);
        $writer->writeU32(0);

        return $writer->finish();
    }

    private static function metadata(int $fields): string
    {
        $writer = new PayloadWriter();
        if (($fields & PropertyFields::NAMES) !== 0) {
            $writer->writeBytes('$value');
        }
        if (($fields & PropertyFields::LOCATIONS) !== 0) {
            $writer->writeBoolean(true);
            $writer->writeBoolean(true);
            $writer->writeBytes('src/Example.php');
            $writer->writeU32(10);
            $writer->writeU32(80);
            $writer->writeBoolean(false);
        }
        if (($fields & PropertyFields::VISIBILITY) !== 0) {
            $writer->writeU8(1);
            $writer->writeU8(3);
        }
        if (($fields & PropertyFields::TYPES) !== 0) {
            foreach ([Type::int(), Type::string(), Type::bool()] as $type) {
                $writer->writeBoolean(true);
                self::location($writer);
                $writer->writeU32(0);
                TypeCodec::writeComplete($writer, $type);
                $writer->writeBoolean(false);
                $writer->writeBoolean(false);
            }
            $writer->writeBoolean(false);
        }
        if (($fields & PropertyFields::ATTRIBUTES) !== 0) {
            $writer->writeU32(1);
            $writer->writeBytes('App\\Field');
            self::location($writer);
            $writer->writeU32(0);
        }
        if (($fields & PropertyFields::FLAGS) !== 0) {
            $writer->writeU64(MetadataFlags::READONLY);
        }
        if (($fields & PropertyFields::HOOKS) !== 0) {
            $writer->writeU32(1);
            $writer->writeBytes('get');
            self::location($writer);
            $writer->writeU64(0);
            $writer->writeBoolean(false);
            $writer->writeBoolean(true);
            $writer->writeBoolean(false);
            $writer->writeU32(0);
            $writer->writeBoolean(false);
            $writer->writeBoolean(false);
        }
        if (($fields & PropertyFields::AVAILABLE_VERSIONS) !== 0) {
            $writer->writeU32(1);
            $writer->writeBoolean(true);
            $writer->writeU32(80_400);
            $writer->writeBoolean(false);
        }

        return $writer->finish();
    }

    private static function location(PayloadWriter $writer): void
    {
        $writer->writeBoolean(false);
        $writer->writeU32(10);
        $writer->writeU32(20);
    }
}
