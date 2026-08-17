<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Worker;

use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Protocol\PayloadReader;
use Mago\Sdk\Internal\Protocol\PayloadWriter;

use function count;
use function pack;
use function strlen;
use function unpack;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 */
final class Protocol
{
    public const COLLECT_REQUEST = 1;
    public const REDUCE_REQUEST = 2;

    private const MAGIC_U32 = 0x4D45_5854;
    private const VERSION_U32 = 0x0001_0000;
    private const COLLECT_RESPONSE = 0x8001;
    private const REDUCE_RESPONSE = 0x8002;
    private const MAXIMUM_WORKERS = 0x0001_0000;
    private const MAXIMUM_REDUCERS = 0x0000_4000;

    /**
     * @return array{int<0, 65535>, PayloadReader}
     */
    public static function readRequest(string $payload): array
    {
        [$kind, $reader] = self::readMessage($payload);
        if ($kind !== self::COLLECT_REQUEST && $kind !== self::REDUCE_REQUEST) {
            throw new ProtocolException("Unknown worker management request kind {$kind}.");
        }

        return [$kind, $reader];
    }

    /**
     * @param array<int<0, max>, string> $payloadsByExtension
     */
    public static function writeCollectResponse(array $payloadsByExtension): string
    {
        $writer = self::createMessage(self::COLLECT_RESPONSE);
        $writer->writeCount($payloadsByExtension);
        foreach ($payloadsByExtension as $extensionIndex => $payload) {
            $writer->writeU32($extensionIndex);
            $writer->writeBytes($payload);
        }

        return $writer->finish();
    }

    /**
     * @param list<int<0, max>> $reducerIndices
     *
     * @return array<int<0, max>, non-empty-list<string>>
     */
    public static function readReduceRequest(PayloadReader $reader, array $reducerIndices): array
    {
        $workerCount = $reader->readCount(self::MAXIMUM_WORKERS);
        if ($workerCount === 0) {
            throw new ProtocolException('A worker reduction request contains no worker payloads.');
        }

        $payloadsByExtension = [];
        for ($workerIndex = 0; $workerIndex < $workerCount; ++$workerIndex) {
            $workerPayloads = self::readCollectResponse($reader->readBytes(), $reducerIndices);
            foreach ($workerPayloads as $extensionIndex => $payload) {
                $payloadsByExtension[$extensionIndex][] = $payload;
            }
        }

        $reader->finish();

        return $payloadsByExtension;
    }

    public static function writeReduceResponse(): string
    {
        return self::createMessage(self::REDUCE_RESPONSE)->finish();
    }

    /**
     * @param list<int<0, max>> $reducerIndices
     *
     * @return array<int<0, max>, string>
     */
    private static function readCollectResponse(string $payload, array $reducerIndices): array
    {
        [$kind, $reader] = self::readMessage($payload);
        if ($kind !== self::COLLECT_RESPONSE) {
            throw new ProtocolException("Expected worker collection response, received message kind {$kind}.");
        }

        $count = $reader->readCount(self::MAXIMUM_REDUCERS);
        if ($count !== count($reducerIndices)) {
            throw new ProtocolException('Workers advertised inconsistent reducer registrations.');
        }

        $payloads = [];
        for ($index = 0; $index < $count; ++$index) {
            $extensionIndex = $reader->readU32();
            if ($extensionIndex !== $reducerIndices[$index]) {
                throw new ProtocolException('Workers advertised inconsistent reducer registrations.');
            }

            $payloads[$extensionIndex] = $reader->readBytes();
        }

        $reader->finish();

        return $payloads;
    }

    /**
     * @return array{int<0, 65535>, PayloadReader}
     */
    private static function readMessage(string $payload): array
    {
        if (strlen($payload) < 12) {
            throw new ProtocolException('Worker management payload is shorter than its header.');
        }

        /** @var array{1: int<0, 4294967295>, 2: int<0, 4294967295>, 3: int<0, 4294967295>} $header */
        $header = unpack('N3', $payload);
        if ($header[1] !== self::MAGIC_U32) {
            throw new ProtocolException('Invalid worker management message magic.');
        }

        if ($header[2] !== self::VERSION_U32) {
            $major = $header[2] >> 16;
            $minor = $header[2] & 0xffff;
            throw new ProtocolException("Unsupported worker management protocol version {$major}.{$minor}.");
        }

        $message = $header[3];
        $reserved = $message & 0xffff;
        if ($reserved !== 0) {
            throw new ProtocolException("Worker management message reserved bits are non-zero: {$reserved}.");
        }

        return [$message >> 16, new PayloadReader($payload, 12)];
    }

    private static function createMessage(int $kind): PayloadWriter
    {
        return new PayloadWriter(pack('N3', self::MAGIC_U32, self::VERSION_U32, $kind << 16));
    }
}
