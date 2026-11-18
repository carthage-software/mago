<?php

declare(strict_types=1);

namespace Mago\Tests\Sdk\Unit\Analyzer;

use Closure;
use Mago\Sdk\Analyzer\Codebase;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Analyzer\MetadataCache;
use Mago\Sdk\Internal\Analyzer\Protocol;
use Mago\Sdk\Internal\HostClient;
use Mago\Sdk\Internal\Io\ResourceWriter;
use Mago\Sdk\Internal\Protocol\Frame;
use Mago\Sdk\Internal\Protocol\FrameCodec;
use Mago\Sdk\Internal\Protocol\FrameKind;
use Mago\Sdk\Internal\Protocol\PayloadWriter;
use Mago\Sdk\Internal\SignalCancellationToken;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;
use ReflectionProperty;
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

/**
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:too-many-methods
 */
final class MetadataCacheConcurrencyTest extends TestCase
{
    public function testMetadataResultsFromAnotherSuspendedRequestAreRetained(): void
    {
        self::withHost(static function (HostClient $host): void {
            $cache = new MetadataCache(7);
            $a = new Codebase($host, 101, new SignalCancellationToken(), $cache);
            $b = new Codebase($host, 102, new SignalCancellationToken(), $cache);
            self::interleave(
                $host,
                static function () use ($a): void {
                    self::assertNull($a->getFunction('A'));
                },
                static function () use ($b): void {
                    self::assertNull($b->getFunction('B'));
                },
                self::missingFunctionResponse(),
                self::missingFunctionResponse(),
            );

            // Assert before querying again so a regression fails rather than hanging without a host response.
            // Null is a valid cached negative lookup, not a missing cache entry.
            $bucket = $cache->values[Protocol::GET_FUNCTIONS << 4] ?? [];
            self::assertArrayHasKey('a', $bucket);
            self::assertArrayHasKey('b', $bucket, 'The later A response must not overwrite the completed B lookup.');
            self::assertNull($b->getFunction('B'));
            self::assertSame(2, (new ReflectionProperty($host, 'nextRequestId'))->getValue($host));
        });
    }

    public function testRelationsFromAnotherSuspendedRequestAreRetained(): void
    {
        self::withHost(static function (HostClient $host): void {
            $cache = new MetadataCache(7);
            $a = new Codebase($host, 101, new SignalCancellationToken(), $cache);
            $b = new Codebase($host, 102, new SignalCancellationToken(), $cache);
            self::interleave(
                $host,
                static function () use ($a): void {
                    self::assertSame(['AChild'], $a->getDirectClassDescendants('A'));
                },
                static function () use ($b): void {
                    self::assertSame(['BChild'], $b->getDirectClassDescendants('B'));
                },
                self::relationResponse('AChild'),
                self::relationResponse('BChild'),
            );

            $bucket = $cache->relations[Protocol::DIRECT_DESCENDANTS] ?? [];
            self::assertSame(['AChild'], $bucket['a'] ?? null);
            self::assertArrayHasKey('b', $bucket, 'The later A response must not overwrite the completed B relation.');
            self::assertSame(['BChild'], $b->getDirectClassDescendants('B'));
            self::assertSame(2, (new ReflectionProperty($host, 'nextRequestId'))->getValue($host));
        });
    }

    /** @return iterable<string, array{string}> */
    public static function existenceKinds(): iterable
    {
        yield 'named symbol' => ['symbol'];
        yield 'class member' => ['member'];
    }

    #[DataProvider('existenceKinds')]
    public function testExistenceResultsFromAnotherSuspendedRequestAreRetained(string $kind): void
    {
        self::withHost(static function (HostClient $host) use ($kind): void {
            $cache = new MetadataCache(7);
            $a = new Codebase($host, 101, new SignalCancellationToken(), $cache);
            $b = new Codebase($host, 102, new SignalCancellationToken(), $cache);
            $lookup = static fn(Codebase $codebase, string $name): bool => $kind === 'member'
                ? $codebase->methodExists('Example', $name)
                : $codebase->functionExists($name);
            self::interleave(
                $host,
                static function () use ($lookup, $a): void {
                    self::assertTrue($lookup($a, 'A'));
                },
                static function () use ($lookup, $b): void {
                    self::assertFalse($lookup($b, 'B'));
                },
                self::existenceResponse($kind, true),
                self::existenceResponse($kind, false),
            );
            $bucketId = $kind === 'member' ? 256 | Protocol::EXISTS_METHOD : Protocol::EXISTS_FUNCTION;
            $key = $kind === 'member' ? "example\0b" : 'b';
            $bucket = $cache->existence[$bucketId] ?? [];
            self::assertArrayHasKey($key, $bucket, 'A must not discard the negative existence result from B.');
            self::assertFalse($lookup($b, 'B'));
            self::assertSame(2, (new ReflectionProperty($host, 'nextRequestId'))->getValue($host));
        });
    }

    /** @return iterable<string, array{string, string}> */
    public static function malformedKinds(): iterable
    {
        foreach (['metadata', 'relations', 'symbol', 'member'] as $kind) {
            yield $kind . '-distinct-keys' => [$kind, 'A'];
            yield $kind . '-overlapping-key' => [$kind, 'B'];
        }
    }

    #[DataProvider('malformedKinds')]
    public function testMalformedLateResponseDoesNotModifyCompletedCache(string $kind, string $lateName): void
    {
        self::withHost(static function (HostClient $host) use ($kind, $lateName): void {
            $cache = new MetadataCache(7);
            $a = new Codebase($host, 101, new SignalCancellationToken(), $cache);
            $b = new Codebase($host, 102, new SignalCancellationToken(), $cache);
            $completedCache = null;
            $response = match ($kind) {
                'metadata' => self::missingFunctionResponse(),
                'relations' => self::relationResponse('Child'),
                default => self::existenceResponse($kind, false),
            };
            self::interleave(
                $host,
                static function () use ($a, $kind, $lateName): void {
                    try {
                        self::assertQueryResult($a, $kind, $lateName);
                        self::fail('A response with trailing bytes was accepted.');
                    } catch (ProtocolException $exception) {
                        self::assertSame('A protocol payload was not consumed exactly.', $exception->getMessage());
                    }
                },
                static function () use ($b, $kind, $cache, &$completedCache): void {
                    self::assertQueryResult($b, $kind, 'B');
                    $completedCache = clone $cache;
                },
                $response . "\0",
                $response,
            );
            self::assertNotNull($completedCache);
            self::assertEquals($completedCache, $cache, 'Malformed A must leave the complete B cache unchanged.');
            self::assertQueryResult($b, $kind, 'B');
            self::assertSame(2, (new ReflectionProperty($host, 'nextRequestId'))->getValue($host));
        });
    }

    private static function assertQueryResult(Codebase $codebase, string $kind, string $name): void
    {
        if ($kind === 'metadata') {
            self::assertNull($codebase->getFunction($name));
            return;
        }
        if ($kind === 'relations') {
            self::assertSame(['Child'], $codebase->getDirectClassDescendants($name));
            return;
        }
        if ($kind === 'member') {
            self::assertFalse($codebase->methodExists('Example', $name));
            return;
        }
        self::assertFalse($codebase->functionExists($name));
    }

    private static function existenceResponse(string $kind, bool $exists): string
    {
        $writer = self::responseHeader(
            $kind === 'member' ? Protocol::CHECK_MEMBER_EXISTENCE : Protocol::CHECK_EXISTENCE,
        );
        $writer->writeU8($kind === 'member' ? Protocol::EXISTS_METHOD : Protocol::EXISTS_FUNCTION);
        $writer->writeU32(1);
        $writer->writeBoolean($exists);
        return $writer->finish();
    }

    /**
     * Revolt runs each callback in its own Fiber. Both lookups suspend in HostClient::request.
     * Deliver B first and only deliver A after B's callback has returned, including its cache write.
     *
     * @param Closure(): void $queryA
     * @param Closure(): void $queryB
     */
    private static function interleave(
        HostClient $host,
        Closure $queryA,
        Closure $queryB,
        string $responseA,
        string $responseB,
    ): void {
        $events = [];
        EventLoop::queue(static function () use ($queryA, &$events): void {
            $events[] = 'A start';
            $queryA();
            $events[] = 'A complete';
        });
        EventLoop::queue(static function () use ($queryB, $host, $responseA, &$events): void {
            $events[] = 'B start';
            $queryB();
            $events[] = 'B complete';
            EventLoop::queue(static function () use ($host, $responseA, &$events): void {
                $events[] = 'A response';
                self::assertTrue($host->accept(new Frame(FrameKind::Response, 0, 1, 101, $responseA)));
            });
        });
        EventLoop::queue(static function () use ($host, $responseB, &$events): void {
            $events[] = 'B response';
            self::assertTrue($host->accept(new Frame(FrameKind::Response, 0, 2, 102, $responseB)));
        });
        EventLoop::run();
        self::assertSame(['A start', 'B start', 'B response', 'B complete', 'A response', 'A complete'], $events);
    }

    private static function responseHeader(int $operation): PayloadWriter
    {
        $writer = new PayloadWriter(pack('N3', 0x4D41_4E41, 0x0002_0002, 0x8004 << 16));
        $writer->writeU64(7);
        $writer->writeU8($operation);
        return $writer;
    }

    private static function missingFunctionResponse(): string
    {
        $writer = self::responseHeader(Protocol::GET_FUNCTIONS);
        $writer->writeU32(1);
        $writer->writeBoolean(false);
        return $writer->finish();
    }

    private static function relationResponse(string $child): string
    {
        $writer = self::responseHeader(Protocol::GET_CLASS_LIKE_RELATIONS);
        $writer->writeU8(Protocol::DIRECT_DESCENDANTS);
        $writer->writeU32(1);
        $writer->writeU32(1);
        $writer->writeBytes($child);
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
