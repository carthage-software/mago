<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Closure;
use Mago\Sdk\Analyzer\Codebase;
use Mago\Sdk\Analyzer\Metadata\MemberIdentifier;
use Mago\Sdk\Analyzer\Metadata\PropertyFields;
use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Analyzer\MetadataCache;
use Mago\Sdk\Internal\HostClient;
use Mago\Sdk\Internal\Io\ResourceWriter;
use Mago\Sdk\Internal\Protocol\Frame;
use Mago\Sdk\Internal\Protocol\FrameCodec;
use Mago\Sdk\Internal\Protocol\FrameKind;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use Mago\Sdk\Internal\SignalCancellationToken;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;
use Revolt\EventLoop;
use RuntimeException;

use function fclose;
use function pack;
use function stream_socket_pair;

use const PHP_OS_FAMILY;
use const STREAM_IPPROTO_IP;
use const STREAM_PF_INET;
use const STREAM_PF_UNIX;
use const STREAM_SOCK_STREAM;

final class PropertyProjectionCacheTest extends TestCase
{
    public function testQueriesPreserveOrderNullsAndCaseSensitiveMembers(): void
    {
        self::withHost(static function (HostClient $host): void {
            $cache = new MetadataCache(7);
            $codebase = new Codebase($host, 1, new SignalCancellationToken(), $cache);
            $name = new MemberIdentifier('App\\A', '$Name');
            $lowerName = new MemberIdentifier('App\\A', '$name');
            $missing = new MemberIdentifier('App\\A', '$missing');
            self::respond($host, 1, self::response(7, PropertyFields::NAMES, ['$Name', null, '$name']));
            $result = $codebase->getMultiplePropertyProjections([
                $name,
                $missing,
                $lowerName,
                $name,
            ], PropertyFields::NAMES);

            self::assertCount(4, $result);
            self::assertSame('$Name', $result[0]?->name);
            self::assertNull($result[1]);
            self::assertSame('$name', $result[2]?->name);
            self::assertSame($result[0], $result[3]);
            self::assertSame(
                [$result[2], null, $result[0]],
                $codebase->getMultiplePropertyProjections([
                    $lowerName,
                    $missing,
                    new MemberIdentifier('app\\a', '$Name'),
                ], PropertyFields::NAMES),
            );
            self::assertSame([], $codebase->getMultiplePropertyProjections([]));
        });
    }

    public function testCacheIsSeparatedByFieldsAndGeneration(): void
    {
        self::withHost(static function (HostClient $host): void {
            $property = new MemberIdentifier('App\\A', '$value');
            $token = new SignalCancellationToken();
            $codebase = new Codebase($host, 1, $token, new MetadataCache(7));
            self::respond($host, 1, self::response(7, PropertyFields::NAMES, ['$value']));
            $named = $codebase->getMultiplePropertyProjections([$property], PropertyFields::NAMES)[0];
            self::assertSame('$value', $named?->name);

            self::respond($host, 2, self::response(7, 0, ['$value']));
            $empty = $codebase->getMultiplePropertyProjections([$property], 0)[0];
            self::assertNotNull($empty);
            self::assertNull($empty->name);
            self::assertFalse($empty->has(PropertyFields::NAMES));
            self::assertSame($named, $codebase->getMultiplePropertyProjections([$property], PropertyFields::NAMES)[0]);

            $nextGeneration = new Codebase($host, 1, $token, new MetadataCache(8));
            self::respond($host, 3, self::response(8, PropertyFields::NAMES, [null]));
            self::assertSame(
                [null],
                $nextGeneration->getMultiplePropertyProjections([$property], PropertyFields::NAMES),
            );
        });
    }

    /** @return iterable<string, array{int}> */
    public static function invalidFields(): iterable
    {
        yield 'negative' => [-1];
        yield 'unknown bit' => [PropertyFields::ALL + 1];
    }

    #[DataProvider('invalidFields')]
    public function testUnknownFieldsAreRejectedEvenForEmptyRequests(int $fields): void
    {
        self::withHost(function (HostClient $host) use ($fields): void {
            $codebase = new Codebase($host, 1, new SignalCancellationToken(), new MetadataCache(7));
            $this->expectException(InvalidArgumentException::class);
            $codebase->getMultiplePropertyProjections([], $fields);
        });
    }

    /** @return iterable<string, array{string}> */
    public static function malformedResults(): iterable
    {
        yield 'wrong count' => [self::response(7, PropertyFields::NAMES, [])];
        yield 'trailing bytes' => [self::response(7, PropertyFields::NAMES, ['$value']) . "\0"];
    }

    #[DataProvider('malformedResults')]
    public function testMalformedResultsAreNotCached(string $payload): void
    {
        self::withHost(static function (HostClient $host) use ($payload): void {
            $cache = new MetadataCache(7);
            $codebase = new Codebase($host, 1, new SignalCancellationToken(), $cache);
            self::respond($host, 1, $payload);
            try {
                $codebase->getMultiplePropertyProjections([new MemberIdentifier(
                    'App\\A',
                    '$value',
                )], PropertyFields::NAMES);
                self::fail('Malformed response was accepted.');
            } catch (ProtocolException) {
                self::assertSame([], $cache->propertyProjections);
            }
        });
    }

    /** @param non-negative-int $id */
    private static function respond(HostClient $host, int $id, string $payload): void
    {
        EventLoop::queue(static function () use ($host, $id, $payload): void {
            self::assertTrue($host->accept(new Frame(FrameKind::Response, 0, $id, 1, $payload)));
        });
    }

    /** @param list<string|null> $names */
    private static function response(int $generation, int $fields, array $names): string
    {
        $writer = new PayloadWriter(pack('N3', 0x4D41_4E41, 0x0002_0002, 0x8004 << 16));
        $writer->writeU64($generation);
        $writer->writeU8(20);
        $writer->writeU32($fields);
        $writer->writeCount($names);
        foreach ($names as $name) {
            $writer->writeBoolean($name !== null);
            if ($name !== null && ($fields & PropertyFields::NAMES) !== 0) {
                $writer->writeBytes($name);
            }
        }

        return $writer->finish();
    }

    /** @param Closure(HostClient): void $test */
    private static function withHost(Closure $test): void
    {
        $domain = PHP_OS_FAMILY === 'Windows' ? STREAM_PF_INET : STREAM_PF_UNIX;
        $streams = stream_socket_pair($domain, STREAM_SOCK_STREAM, STREAM_IPPROTO_IP);
        if ($streams === false) {
            throw new RuntimeException('Unable to create SDK test streams.');
        }
        [$output, $input] = $streams;
        $writer = new ResourceWriter($output);
        try {
            $test(new HostClient(new FrameCodec(), $writer));
        } finally {
            $writer->close();
            fclose($output);
            fclose($input);
        }
    }
}
