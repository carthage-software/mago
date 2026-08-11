<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Protocol;

use Mago\Sdk\Exception\ProtocolException;

use function ord;
use function strlen;
use function substr;
use function unpack;

/**
 * @internal
 * @mago-expect lint:cyclomatic-complexity
 * @mago-expect lint:too-many-methods
 */
final class PayloadReader
{
    /**
     * @var non-negative-int
     */
    private int $offset;

    /**
     * @var non-negative-int
     */
    private readonly int $length;

    /**
     * @param non-negative-int $offset
     */
    public function __construct(
        private readonly string $payload,
        int $offset = 0,
    ) {
        $this->offset = $offset;
        $this->length = strlen($payload);
    }

    /**
     * @return int<0, 255>
     */
    public function readU8(): int
    {
        $value = ord($this->payload[$this->offset]);
        ++$this->offset;

        return $value;
    }

    /**
     * @return int<0, 65535>
     */
    public function readU16(): int
    {
        /** @var array{1: int<0, 65535>} $decoded */
        $decoded = unpack('n', $this->payload, $this->offset);
        $value = $decoded[1];
        $this->offset += 2;

        return $value;
    }

    /**
     * @return int<0, 4294967295>
     */
    public function readU32(): int
    {
        /** @var array{1: int<0, 4294967295>} $decoded */
        $decoded = unpack('N', $this->payload, $this->offset);
        $value = $decoded[1];
        $this->offset += 4;

        return $value;
    }

    /**
     * @return int<0, max>
     */
    public function readU64(): int
    {
        /** @var array{1: int} $decoded */
        $decoded = unpack('J', $this->payload, $this->offset);
        $value = $decoded[1];
        $this->offset += 8;
        if ($value < 0) {
            throw new ProtocolException('An unsigned protocol integer exceeds the largest integer supported by PHP.');
        }

        return $value;
    }

    /**
     * @param int<0, max> $count
     *
     * @return array<int, int<0, 4294967295>>
     */
    public function readU32List(int $count): array
    {
        if ($count === 0) {
            return [];
        }

        if ($count === 1) {
            return [$this->readU32()];
        }

        $length = $count * 4;
        /** @var array<int<1, max>, int<0, 4294967295>> $values */
        $values = unpack('N' . $count, $this->payload, $this->offset);
        $this->offset += $length;

        return $values;
    }

    public function readBoolean(): bool
    {
        return match ($value = $this->readU8()) {
            0 => false,
            1 => true,
            default => throw new ProtocolException("A protocol boolean contains invalid value {$value}."),
        };
    }

    /**
     * @param int<0, 4294967295> $maximum
     *
     * @return int<0, 4294967295>
     */
    public function readCount(int $maximum): int
    {
        /** @var array{1: int<0, 4294967295>} $decoded */
        $decoded = unpack('N', $this->payload, $this->offset);
        $count = $decoded[1];
        $this->offset += 4;
        if ($count > $maximum) {
            throw new ProtocolException("A protocol collection count {$count} exceeds the limit of {$maximum}.");
        }

        return $count;
    }

    /**
     * @param int<0, max> $length
     */
    public function readRaw(int $length): string
    {
        if ($length === 0) {
            return '';
        }

        $value = substr($this->payload, $this->offset, $length);
        $this->offset += $length;

        return $value;
    }

    public function readBytes(): string
    {
        /** @var array{1: int<0, 4294967295>} $decoded */
        $decoded = unpack('N', $this->payload, $this->offset);
        $length = $decoded[1];
        $this->offset += 4;
        if ($length === 0) {
            return '';
        }

        $value = substr($this->payload, $this->offset, $length);
        $this->offset += $length;

        return $value;
    }

    public function readString(): string
    {
        /** @var array{1: int<0, 4294967295>} $decoded */
        $decoded = unpack('N', $this->payload, $this->offset);
        $length = $decoded[1];
        $this->offset += 4;
        if ($length === 0) {
            return '';
        }

        $value = substr($this->payload, $this->offset, $length);
        $this->offset += $length;

        return $value;
    }

    public function readOptionalString(): ?string
    {
        return $this->readBoolean() ? $this->readString() : null;
    }

    public function finish(): void
    {
        if ($this->offset !== $this->length) {
            throw new ProtocolException('A protocol payload was not consumed exactly.');
        }
    }
}
