<?php

declare(strict_types=1);

namespace Mago\Sdk\Internal\Protocol;

use function chr;
use function count;
use function pack;
use function strlen;

/**
 * @internal
 * @mago-expect lint:too-many-methods
 */
final class PayloadWriter
{
    public function __construct(
        private string $payload = '',
    ) {}

    public function writeRaw(string $value): void
    {
        $this->payload .= $value;
    }

    public function writeU8(int $value): void
    {
        $this->payload .= chr($value);
    }

    public function writeU16(int $value): void
    {
        $this->payload .= pack('n', $value);
    }

    public function writeU32(int $value): void
    {
        $this->payload .= pack('N', $value);
    }

    public function writeU64(int $value): void
    {
        $this->payload .= pack('J', $value);
    }

    public function writeI64(int $value): void
    {
        $this->payload .= pack('J', $value);
    }

    public function writeF64(float $value): void
    {
        $this->payload .= pack('E', $value);
    }

    /**
     * @mago-expect lint:no-boolean-flag-parameter
     */
    public function writeBoolean(bool $value): void
    {
        $this->payload .= $value ? "\1" : "\0";
    }

    /**
     * @param array<array-key, mixed> $values
     */
    public function writeCount(array $values): void
    {
        $this->payload .= pack('N', count($values));
    }

    public function writeLength(int $length): void
    {
        $this->payload .= pack('N', $length);
    }

    public function writeBytes(string $value): void
    {
        $this->payload .= pack('N', strlen($value)) . $value;
    }

    public function writeString(string $value): void
    {
        $this->payload .= pack('N', strlen($value)) . $value;
    }

    public function writeOptionalString(?string $value): void
    {
        if ($value === null) {
            $this->payload .= "\0";
            return;
        }

        $this->payload .= "\1" . pack('N', strlen($value)) . $value;
    }

    public function finish(): string
    {
        return $this->payload;
    }
}
