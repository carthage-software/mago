<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Protocol;

use Mago\Sdk\Exception\InvalidArgumentException;
use Mago\Sdk\Exception\ProtocolException;
use Mago\Sdk\Internal\Io\ResourceReader;

use function pack;
use function strlen;
use function unpack;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 */
final class FrameCodec
{
    public const DEFAULT_MAXIMUM_PAYLOAD_SIZE = 67_108_864;

    private const MAGIC = 'MAGO';
    private const MAGIC_U32 = 0x4D41_474F;
    private const HEADER_LENGTH = 32;
    private const PROTOCOL_MAJOR = 1;
    private const PROTOCOL_MINOR = 0;
    private const VERSION_U32 = (self::PROTOCOL_MAJOR << 16) | self::PROTOCOL_MINOR;
    private const RESPONSE_KIND = 2;

    public function __construct(
        private readonly int $maximumPayloadSize = self::DEFAULT_MAXIMUM_PAYLOAD_SIZE,
    ) {
        if ($maximumPayloadSize < 1 || $maximumPayloadSize > 4_294_967_295) {
            throw new InvalidArgumentException(
                'The maximum payload size must fit in a non-zero unsigned 32-bit integer.',
            );
        }
    }

    public function read(ResourceReader $input): ?Frame
    {
        $header = $input->readExactly(self::HEADER_LENGTH);
        if ($header === null) {
            return null;
        }

        /**
         * @var array{
         *   1: int<0, 4294967295>,
         *   2: int<0, 4294967295>,
         *   3: int<0, 4294967295>,
         *   4: int<0, 4294967295>,
         *   5: int<0, 4294967295>,
         *   6: int<0, 4294967295>,
         *   7: int<0, 4294967295>,
         *   8: int<0, 4294967295>
         * } $fields
         **/
        $fields = unpack('N8', $header);
        if ($fields[1] !== self::MAGIC_U32) {
            throw new ProtocolException('Invalid extension frame magic.');
        }

        $version = $fields[2];
        if ($version !== self::VERSION_U32) {
            $major = $version >> 16;
            $minor = $version & 0xffff;
            throw new ProtocolException("Unsupported extension frame protocol version {$major}.{$minor}.");
        }

        $metadata = $fields[3];
        $kindValue = $metadata >> 24;
        $kind = FrameKind::tryFrom($kindValue);
        if ($kind === null) {
            throw new ProtocolException("Unknown extension frame kind {$kindValue}.");
        }

        $flags = ($metadata >> 16) & 0xff;
        $reserved = $metadata & 0xffff;
        if ($reserved !== 0) {
            throw new ProtocolException("Extension frame reserved bits are non-zero: {$reserved}.");
        }

        /** @var int $id */
        $id = ($fields[4] << 32) | $fields[5];
        /** @var int $parentId */
        $parentId = ($fields[6] << 32) | $fields[7];
        if ($id < 0 || $parentId < 0) {
            throw new ProtocolException('An extension frame identifier exceeds the largest integer supported by PHP.');
        }

        $payloadLength = $fields[8];
        if ($payloadLength > $this->maximumPayloadSize) {
            throw new ProtocolException(
                "Extension frame is {$payloadLength} bytes, exceeding the {$this->maximumPayloadSize}-byte limit.",
            );
        }

        if ($payloadLength === 0) {
            return new Frame($kind, $flags, $id, $parentId, '');
        }

        $payload = $input->readExactly($payloadLength);
        if ($payload === null) {
            throw new ProtocolException('The Mago worker input ended before the frame payload was received.');
        }

        return new Frame($kind, $flags, $id, $parentId, $payload);
    }

    public function encode(Frame $frame): string
    {
        $payloadLength = strlen($frame->payload);
        if ($payloadLength > $this->maximumPayloadSize) {
            throw new ProtocolException(
                "Extension frame is {$payloadLength} bytes, exceeding the {$this->maximumPayloadSize}-byte limit.",
            );
        }

        return (
            pack(
                'a4nnCCnJJN',
                self::MAGIC,
                self::PROTOCOL_MAJOR,
                self::PROTOCOL_MINOR,
                $frame->kind->value,
                $frame->flags,
                0,
                $frame->id,
                $frame->parentId,
                $payloadLength,
            ) . $frame->payload
        );
    }

    /**
     * @param int<0, max> $id
     * @param int<0, 255> $flags
     */
    public function encodeResponse(int $id, string $payload, int $flags = 0): string
    {
        $payloadLength = strlen($payload);
        if ($payloadLength > $this->maximumPayloadSize) {
            throw new ProtocolException(
                "Extension frame is {$payloadLength} bytes, exceeding the {$this->maximumPayloadSize}-byte limit.",
            );
        }

        return (
            pack(
                'a4nnCCnJJN',
                self::MAGIC,
                self::PROTOCOL_MAJOR,
                self::PROTOCOL_MINOR,
                self::RESPONSE_KIND,
                $flags,
                0,
                $id,
                0,
                $payloadLength,
            ) . $payload
        );
    }
}
